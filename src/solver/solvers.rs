use std::collections::HashSet;
use swc_common::{FileName, SourceMap, sync::Lrc};
use swc_ecma_ast::*;
use swc_ecma_codegen::{Config, Emitter, text_writer::JsWriter};
use swc_ecma_parser::{Parser, StringInput, Syntax, lexer::Lexer};

use super::n;
use super::setup::{INTL_POLYFILL, SETUP_CODE};
use super::sig;

/// Preprocess YouTube player code to extract sig and n functions
pub fn preprocess_player(data: &str) -> Result<String, String> {
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
        .map_err(|e| format!("Parse error: {:?}", e))?;

    // Extract the main block from IIFE structure
    let block = extract_main_block(&module)?;

    // Find n and sig functions
    let mut found_n: Vec<String> = Vec::new();
    let mut found_sig: Vec<String> = Vec::new();
    let mut filtered_stmts: Vec<Stmt> = Vec::new();

    for stmt in &block.stmts {
        // Try to extract n function
        if let Some(n_func) = n::extract(stmt) {
            found_n.push(n_func);
        }

        // Try to extract sig function
        if let Some(sig_func) = sig::extract(stmt) {
            found_sig.push(sig_func);
        }

        // Transform `g.XX = this || self` to `g.XX = self`
        let stmt = transform_this_or_self(stmt);

        // Keep expression statements and declarations
        // Convert `function g(...)` to `g = function(...)` to avoid conflict with `var g = {}`
        match &stmt {
            Stmt::Decl(Decl::Fn(fn_decl)) if &*fn_decl.ident.sym == "g" => {
                // Convert function declaration to assignment expression
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
            Stmt::Expr(expr_stmt) => match &*expr_stmt.expr {
                Expr::Assign(_) | Expr::Lit(_) => {
                    filtered_stmts.push(stmt.clone());
                }
                _ => {
                    filtered_stmts.push(stmt.clone());
                }
            },
            _ => {
                filtered_stmts.push(stmt.clone());
            }
        }
    }

    // Validate unique functions found
    let unique_n: HashSet<_> = found_n.iter().collect();
    let unique_sig: HashSet<_> = found_sig.iter().collect();

    if unique_n.len() != 1 {
        return Err(format!(
            "found {} n function possibilities: {:?}",
            unique_n.len(),
            found_n
        ));
    }

    if unique_sig.len() != 1 {
        return Err(format!(
            "found {} sig function possibilities: {:?}",
            unique_sig.len(),
            found_sig
        ));
    }

    // Generate output code
    let n_func = &found_n[0];
    let sig_func = &found_sig[0];

    // Generate the filtered module code
    let filtered_module = Module {
        span: module.span,
        body: filtered_stmts.into_iter().map(ModuleItem::Stmt).collect(),
        shebang: None,
    };

    let module_code = generate_code(&cm, &filtered_module);

    // Combine setup code, module code, and result assignments
    let result = format!(
        "{}\n{}\n{}\n_result.n = {};\n_result.sig = {};",
        INTL_POLYFILL, SETUP_CODE, module_code, n_func, sig_func
    );

    Ok(result)
}

fn extract_main_block(module: &Module) -> Result<BlockStmt, String> {
    match module.body.len() {
        1 => {
            // Pattern: (function() { ... }).call(this)
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
                // Also try Paren wrapped function
                if let Expr::Paren(paren) = &*member.obj
                    && let Expr::Fn(fn_expr) = &*paren.expr
                    && let Some(body) = &fn_expr.function.body
                {
                    return Ok(body.clone());
                }
            }
            Err("unexpected structure (single item)".into())
        }
        2 => {
            // Pattern 1: var _yt_player={}; (function(g){...}).call(this)
            // Pattern 2: 'use strict'; (function() { ... })()
            let item = &module.body[1];
            if let ModuleItem::Stmt(Stmt::Expr(expr_stmt)) = item
                && let Expr::Call(call_expr) = &*expr_stmt.expr
                && let Callee::Expr(callee) = &call_expr.callee
            {
                // Pattern 1: (function(g){...}).call(this) - MemberExpression
                if let Expr::Member(member) = &**callee {
                    if let Expr::Fn(fn_expr) = &*member.obj
                        && let Some(body) = &fn_expr.function.body
                    {
                        let mut block = body.clone();
                        // Skip `var window = this;`
                        if !block.stmts.is_empty() {
                            block.stmts.remove(0);
                        }
                        return Ok(block);
                    }
                    // Also try Paren wrapped function
                    if let Expr::Paren(paren) = &*member.obj
                        && let Expr::Fn(fn_expr) = &*paren.expr
                        && let Some(body) = &fn_expr.function.body
                    {
                        let mut block = body.clone();
                        // Skip `var window = this;`
                        if !block.stmts.is_empty() {
                            block.stmts.remove(0);
                        }
                        return Ok(block);
                    }
                }
                // Pattern 2: (function() { ... })() - direct call
                if let Expr::Fn(fn_expr) = &**callee
                    && let Some(body) = &fn_expr.function.body
                {
                    let mut block = body.clone();
                    // Skip `var window = this;`
                    if !block.stmts.is_empty() {
                        block.stmts.remove(0);
                    }
                    return Ok(block);
                }
                // Also try Paren wrapped function for direct call
                if let Expr::Paren(paren) = &**callee {
                    if let Expr::Fn(fn_expr) = &*paren.expr
                        && let Some(body) = &fn_expr.function.body
                    {
                        let mut block = body.clone();
                        // Skip `var window = this;`
                        if !block.stmts.is_empty() {
                            block.stmts.remove(0);
                        }
                        return Ok(block);
                    }
                    // Pattern: ((function(g){...}).call(this)) - Paren wrapping Call with Member
                    if let Expr::Call(inner_call) = &*paren.expr
                        && let Callee::Expr(inner_callee) = &inner_call.callee
                        && let Expr::Member(member) = &**inner_callee
                        && let Expr::Fn(fn_expr) = &*member.obj
                        && let Some(body) = &fn_expr.function.body
                    {
                        let mut block = body.clone();
                        // Skip `var window = this;`
                        if !block.stmts.is_empty() {
                            block.stmts.remove(0);
                        }
                        return Ok(block);
                    }
                }
            }
            Err("unexpected structure (two items)".into())
        }
        _ => Err(format!("unexpected structure: {} items", module.body.len())),
    }
}

/// Transform `g.XX = this || self` to `g.XX = self`
fn transform_this_or_self(stmt: &Stmt) -> Stmt {
    if let Stmt::Expr(expr_stmt) = stmt
        && let Expr::Assign(assign_expr) = &*expr_stmt.expr
    {
        // Check if right side is `this || self`
        if let Expr::Bin(bin_expr) = &*assign_expr.right
            && bin_expr.op == BinaryOp::LogicalOr
        {
            // Check if left is `this` and right is `self`
            let is_this = matches!(&*bin_expr.left, Expr::This(_));
            let is_self = matches!(&*bin_expr.right, Expr::Ident(ident) if &*ident.sym == "self");

            if is_this && is_self {
                // Create new assignment with just `self`
                let new_assign = Expr::Assign(AssignExpr {
                    span: assign_expr.span,
                    op: assign_expr.op,
                    left: assign_expr.left.clone(),
                    right: bin_expr.right.clone(),
                });
                return Stmt::Expr(ExprStmt {
                    span: expr_stmt.span,
                    expr: Box::new(new_assign),
                });
            }
        }
    }
    stmt.clone()
}

fn generate_code(cm: &Lrc<SourceMap>, module: &Module) -> String {
    let mut buf = vec![];
    {
        let writer = JsWriter::new(cm.clone(), "\n", &mut buf, None);
        let config = Config::default();
        let mut emitter = Emitter {
            cfg: config,
            cm: cm.clone(),
            comments: None,
            wr: writer,
        };
        emitter.emit_module(module).unwrap();
    }
    String::from_utf8(buf).unwrap()
}
