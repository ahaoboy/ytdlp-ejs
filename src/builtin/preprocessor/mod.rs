//! Player Preprocessor

mod n;
mod sig;

use std::collections::HashSet;
use swc_common::{FileName, SourceMap, sync::Lrc};
use swc_ecma_ast::*;
use swc_ecma_codegen::{Config, Emitter, text_writer::JsWriter};
use swc_ecma_parser::{Parser, StringInput, Syntax, lexer::Lexer};

use crate::builtin::polyfill::{INTL_POLYFILL, SETUP_CODE};
use crate::provider::JsChallengeError;

/// Preprocess YouTube player code to extract sig and n functions
pub fn preprocess_player(data: &str) -> Result<String, JsChallengeError> {
    let cm: Lrc<SourceMap> = Default::default();
    let fm = cm.new_source_file(
        FileName::Custom("player.js".into()).into(),
        data.to_string(),
    );

    let lexer = Lexer::new(
        Syntax::Es(Default::default()),
        Default::default(),
        StringInput::from(&*fm),
        None,
    );

    let mut parser = Parser::new_from(lexer);
    let module = parser
        .parse_module()
        .map_err(|e| JsChallengeError::Parse(format!("{:?}", e)))?;

    let block = extract_main_block(&module)?;

    let mut found_n: Vec<String> = Vec::new();
    let mut found_sig: Vec<String> = Vec::new();
    let mut filtered_stmts: Vec<Stmt> = Vec::new();

    for stmt in &block.stmts {
        if let Some(n_func) = n::extract(stmt) {
            found_n.push(n_func);
        }
        if let Some(sig_func) = sig::extract(stmt) {
            found_sig.push(sig_func);
        }

        let stmt = transform_this_or_self(stmt);

        match &stmt {
            Stmt::Decl(Decl::Fn(fn_decl)) if &*fn_decl.ident.sym == "g" => {
                let fn_expr = Expr::Fn(FnExpr {
                    ident: None,
                    function: fn_decl.function.clone(),
                });
                let assign = Expr::Assign(AssignExpr {
                    span: fn_decl.function.span,
                    op: AssignOp::Assign,
                    left: AssignTarget::Simple(SimpleAssignTarget::Ident(BindingIdent {
                        id: fn_decl.ident.clone(),
                        type_ann: None,
                    })),
                    right: Box::new(fn_expr),
                });
                filtered_stmts.push(Stmt::Expr(ExprStmt {
                    span: fn_decl.function.span,
                    expr: Box::new(assign),
                }));
            }
            _ => filtered_stmts.push(stmt.clone()),
        }
    }

    let unique_n: HashSet<_> = found_n.iter().collect();
    let unique_sig: HashSet<_> = found_sig.iter().collect();

    if unique_n.len() != 1 {
        return Err(JsChallengeError::Preprocess(format!(
            "found {} n functions: {:?}",
            unique_n.len(),
            found_n
        )));
    }
    if unique_sig.len() != 1 {
        return Err(JsChallengeError::Preprocess(format!(
            "found {} sig functions: {:?}",
            unique_sig.len(),
            found_sig
        )));
    }

    let n_func = &found_n[0];
    let sig_func = &found_sig[0];

    let filtered_module = Module {
        span: module.span,
        body: filtered_stmts.into_iter().map(ModuleItem::Stmt).collect(),
        shebang: None,
    };

    let module_code = generate_code(&cm, &filtered_module)?;

    Ok(format!(
        "{}\n{}\n{}\n_result.n = {};\n_result.sig = {};",
        INTL_POLYFILL, SETUP_CODE, module_code, n_func, sig_func
    ))
}

fn extract_main_block(module: &Module) -> Result<BlockStmt, JsChallengeError> {
    match module.body.len() {
        1 => {
            let item = &module.body[0];
            if let ModuleItem::Stmt(Stmt::Expr(expr_stmt)) = item
                && let Expr::Call(call_expr) = &*expr_stmt.expr
                && let Callee::Expr(callee) = &call_expr.callee
                && let Expr::Member(member) = &**callee
            {
                if let Expr::Fn(fn_expr) = &*member.obj
                    && let Some(body) = &fn_expr.function.body
                {
                    return Ok(body.clone());
                }
                if let Expr::Paren(paren) = &*member.obj
                    && let Expr::Fn(fn_expr) = &*paren.expr
                    && let Some(body) = &fn_expr.function.body
                {
                    return Ok(body.clone());
                }
            }
            Err(JsChallengeError::Parse(
                "unexpected structure (single item)".into(),
            ))
        }
        2 => {
            let item = &module.body[1];
            if let ModuleItem::Stmt(Stmt::Expr(expr_stmt)) = item
                && let Expr::Call(call_expr) = &*expr_stmt.expr
                && let Callee::Expr(callee) = &call_expr.callee
            {
                if let Expr::Member(member) = &**callee {
                    if let Expr::Fn(fn_expr) = &*member.obj
                        && let Some(body) = &fn_expr.function.body
                    {
                        let mut block = body.clone();
                        if !block.stmts.is_empty() {
                            block.stmts.remove(0);
                        }
                        return Ok(block);
                    }
                    if let Expr::Paren(paren) = &*member.obj
                        && let Expr::Fn(fn_expr) = &*paren.expr
                        && let Some(body) = &fn_expr.function.body
                    {
                        let mut block = body.clone();
                        if !block.stmts.is_empty() {
                            block.stmts.remove(0);
                        }
                        return Ok(block);
                    }
                }
                if let Expr::Fn(fn_expr) = &**callee
                    && let Some(body) = &fn_expr.function.body
                {
                    let mut block = body.clone();
                    if !block.stmts.is_empty() {
                        block.stmts.remove(0);
                    }
                    return Ok(block);
                }
                if let Expr::Paren(paren) = &**callee {
                    if let Expr::Fn(fn_expr) = &*paren.expr
                        && let Some(body) = &fn_expr.function.body
                    {
                        let mut block = body.clone();
                        if !block.stmts.is_empty() {
                            block.stmts.remove(0);
                        }
                        return Ok(block);
                    }
                    if let Expr::Call(inner_call) = &*paren.expr
                        && let Callee::Expr(inner_callee) = &inner_call.callee
                        && let Expr::Member(member) = &**inner_callee
                        && let Expr::Fn(fn_expr) = &*member.obj
                        && let Some(body) = &fn_expr.function.body
                    {
                        let mut block = body.clone();
                        if !block.stmts.is_empty() {
                            block.stmts.remove(0);
                        }
                        return Ok(block);
                    }
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

fn transform_this_or_self(stmt: &Stmt) -> Stmt {
    if let Stmt::Expr(expr_stmt) = stmt
        && let Expr::Assign(assign_expr) = &*expr_stmt.expr
        && let Expr::Bin(bin_expr) = &*assign_expr.right
        && bin_expr.op == BinaryOp::LogicalOr
    {
        let is_this = matches!(&*bin_expr.left, Expr::This(_));
        let is_self = matches!(&*bin_expr.right, Expr::Ident(ident) if &*ident.sym == "self");

        if is_this && is_self {
            return Stmt::Expr(ExprStmt {
                span: expr_stmt.span,
                expr: Box::new(Expr::Assign(AssignExpr {
                    span: assign_expr.span,
                    op: assign_expr.op,
                    left: assign_expr.left.clone(),
                    right: bin_expr.right.clone(),
                })),
            });
        }
    }
    stmt.clone()
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
