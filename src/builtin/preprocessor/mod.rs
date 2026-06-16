//! Player Preprocessor
//!
//! Parses YouTube player JavaScript, extracts the inner function body from
//! the IIFE wrapper, filters statements, locates n/sig solver functions,
//! and generates the final preprocessed code with solver assignments.

mod extract_shared;

use swc_common::{sync::Lrc, FileName, SourceMap, SyntaxContext, DUMMY_SP};
use swc_ecma_ast::*;
use swc_ecma_codegen::{text_writer::JsWriter, Config, Emitter};
use swc_ecma_parser::{lexer::Lexer, Parser, StringInput, Syntax};

use crate::builtin::polyfill::{INTL_POLYFILL, SETUP_CODE};
use crate::provider::JsChallengeError;

/// Preprocess YouTube player code to extract sig and n functions.
/// Returns the final executable preprocessed JavaScript code.
pub fn preprocess_player(data: &str) -> Result<String, JsChallengeError> {
    let cm: Lrc<SourceMap> = Default::default();
    let fm = cm.new_source_file(FileName::Custom("player.js".into()).into(), data.to_string());

    let lexer = Lexer::new(
        Syntax::Es(Default::default()),
        Default::default(),
        StringInput::from(&*fm),
        None,
    );

    let mut parser = Parser::new_from(lexer);
    let mut module = parser
        .parse_module()
        .map_err(|e| JsChallengeError::Parse(format!("{:?}", e)))?;

    // Extract the inner function body from the IIFE wrapper
    let block_stmts = extract_main_block_mut(&mut module)?;

    // Filter: keep only assignments, declarations, literal expressions
    let mut kept = Vec::new();
    for mut stmt in std::mem::take(block_stmts) {
        let should_keep = match &stmt {
            Stmt::Expr(e) => matches!(&*e.expr, Expr::Assign(_) | Expr::Lit(_)),
            _ => true,
        };

        if should_keep {
            transform_this_or_self_mut(&mut stmt);

            // Convert `function g(...)` → `g = function(...)` for QJS compat
            if let Stmt::Decl(Decl::Fn(ref fn_decl)) = stmt
                && &*fn_decl.ident.sym == "g" {
                    let fn_expr = Expr::Fn(FnExpr {
                        ident: None,
                        function: fn_decl.function.clone(),
                    });
                    stmt = Stmt::Expr(ExprStmt {
                        span: fn_decl.function.span,
                        expr: Box::new(Expr::Assign(AssignExpr {
                            span: fn_decl.function.span,
                            op: AssignOp::Assign,
                            left: AssignTarget::Simple(SimpleAssignTarget::Ident(
                                BindingIdent {
                                    id: fn_decl.ident.clone(),
                                    type_ann: None,
                                },
                            )),
                            right: Box::new(fn_expr),
                        })),
                    });
                }
            kept.push(stmt);
        }
    }
    *block_stmts = kept;

    // Extract solvers from block statements
    let mut found_n: Vec<Box<Expr>> = Vec::new();
    let mut found_sig: Vec<Box<Expr>> = Vec::new();

    for stmt in block_stmts.iter() {
        for info in extract_shared::extract_solver_infos(stmt) {
            let solver = extract_shared::generate_solver_expr(&info.name_expr)?;
            found_n.push(extract_shared::generate_n_solver_expr(&solver)?);
            found_sig.push(extract_shared::generate_sig_solver_expr(&solver)?);
        }
    }

    if found_n.is_empty() {
        return Err(JsChallengeError::Preprocess("found 0 n functions".into()));
    }
    if found_sig.is_empty() {
        return Err(JsChallengeError::Preprocess("found 0 sig functions".into()));
    }

    // Add _result.n / _result.sig assignments with multiTry wrappers
    let _result = Ident::new("_result".into(), DUMMY_SP, SyntaxContext::empty());
    block_stmts.push(make_result_assign(&_result, "n", extract_shared::generate_multi_try_expr(&found_n)?));
    block_stmts.push(make_result_assign(&_result, "sig", extract_shared::generate_multi_try_expr(&found_sig)?));

    // Prepend polyfills (browser env shims) to the module body
    let mut polyfills: Vec<ModuleItem> =
        extract_shared::parse_script(INTL_POLYFILL)?
            .into_iter()
            .chain(extract_shared::parse_script(SETUP_CODE)?)
            .map(ModuleItem::Stmt)
            .collect();

    let original_body = std::mem::take(&mut module.body);
    polyfills.extend(original_body);
    module.body = polyfills;

    generate_code(&cm, &module)
}

// ── Helpers ──────────────────────────────────────────────────────────────────

fn make_result_assign(result_ident: &Ident, prop: &str, expr: Box<Expr>) -> Stmt {
    Stmt::Expr(ExprStmt {
        span: DUMMY_SP,
        expr: Box::new(Expr::Assign(AssignExpr {
            span: DUMMY_SP,
            op: AssignOp::Assign,
            left: AssignTarget::Simple(SimpleAssignTarget::Member(MemberExpr {
                span: DUMMY_SP,
                obj: Box::new(Expr::Ident(result_ident.clone())),
                prop: MemberProp::Ident(IdentName::new(prop.into(), DUMMY_SP)),
            })),
            right: expr,
        })),
    })
}

fn extract_main_block_mut(module: &mut Module) -> Result<&mut Vec<Stmt>, JsChallengeError> {
    match module.body.len() {
        1 => {
            let item = &mut module.body[0];
            if let ModuleItem::Stmt(Stmt::Expr(expr_stmt)) = item
                && let Expr::Call(call_expr) = &mut *expr_stmt.expr
                && let Callee::Expr(callee) = &mut call_expr.callee
                && let Expr::Member(member) = &mut **callee
            {
                match &mut *member.obj {
                    Expr::Fn(fn_expr) => {
                        if let Some(body) = &mut fn_expr.function.body {
                            return Ok(&mut body.stmts);
                        }
                    }
                    Expr::Paren(paren) => {
                        if let Expr::Fn(fn_expr) = &mut *paren.expr
                            && let Some(body) = &mut fn_expr.function.body {
                                return Ok(&mut body.stmts);
                            }
                    }
                    _ => {}
                }
            }
            Err(JsChallengeError::Parse(
                "unexpected structure (single item)".into(),
            ))
        }
        2 => {
            let item = &mut module.body[1];
            if let ModuleItem::Stmt(Stmt::Expr(expr_stmt)) = item
                && let Expr::Call(call_expr) = &mut *expr_stmt.expr
                && let Callee::Expr(callee) = &mut call_expr.callee
            {
                match &mut **callee {
                    Expr::Member(member) => {
                        match &mut *member.obj {
                            Expr::Fn(fn_expr) => {
                                if let Some(body) = &mut fn_expr.function.body {
                                    if !body.stmts.is_empty() {
                                        body.stmts.remove(0);
                                    }
                                    return Ok(&mut body.stmts);
                                }
                            }
                            Expr::Paren(paren) => {
                                if let Expr::Fn(fn_expr) = &mut *paren.expr
                                    && let Some(body) = &mut fn_expr.function.body {
                                        if !body.stmts.is_empty() {
                                            body.stmts.remove(0);
                                        }
                                        return Ok(&mut body.stmts);
                                    }
                            }
                            _ => {}
                        }
                    }
                    Expr::Fn(fn_expr) => {
                        if let Some(body) = &mut fn_expr.function.body {
                            if !body.stmts.is_empty() {
                                body.stmts.remove(0);
                            }
                            return Ok(&mut body.stmts);
                        }
                    }
                    Expr::Paren(paren) => {
                        match &mut *paren.expr {
                            Expr::Fn(fn_expr) => {
                                if let Some(body) = &mut fn_expr.function.body {
                                    if !body.stmts.is_empty() {
                                        body.stmts.remove(0);
                                    }
                                    return Ok(&mut body.stmts);
                                }
                            }
                            Expr::Call(inner_call) => {
                                if let Callee::Expr(inner_callee) = &mut inner_call.callee
                                    && let Expr::Member(member) = &mut **inner_callee
                                    && let Expr::Fn(fn_expr) = &mut *member.obj
                                    && let Some(body) = &mut fn_expr.function.body
                                {
                                    if !body.stmts.is_empty() {
                                        body.stmts.remove(0);
                                    }
                                    return Ok(&mut body.stmts);
                                }
                            }
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }
            Err(JsChallengeError::Parse(
                "unexpected structure (two items)".into(),
            ))
        }
        n => Err(JsChallengeError::Parse(format!(
            "unexpected structure: {} items",
            n
        ))),
    }
}

fn transform_this_or_self_mut(stmt: &mut Stmt) {
    if let Stmt::Expr(expr_stmt) = stmt
        && let Expr::Assign(assign_expr) = &mut *expr_stmt.expr
        && let Expr::Bin(bin_expr) = &mut *assign_expr.right
        && bin_expr.op == BinaryOp::LogicalOr
    {
        let is_this = matches!(&*bin_expr.left, Expr::This(_));
        let is_self = matches!(&*bin_expr.right, Expr::Ident(ident) if &*ident.sym == "self");

        if is_this && is_self {
            assign_expr.right = bin_expr.right.clone();
        }
    }
}

fn generate_code(cm: &Lrc<SourceMap>, module: &Module) -> Result<String, JsChallengeError> {
    let mut buf = vec![];
    {
        let writer = JsWriter::new(cm.clone(), "\n", &mut buf, None);
        let mut emitter = Emitter {
            cfg: Config::default(),
            cm: cm.clone(),
            comments: None,
            wr: writer,
        };
        emitter
            .emit_module(module)
            .map_err(|e| JsChallengeError::Runtime(format!("Code generation failed: {}", e)))?;
    }
    String::from_utf8(buf).map_err(|e| JsChallengeError::Runtime(format!("UTF-8 error: {}", e)))
}

