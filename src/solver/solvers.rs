use std::collections::HashSet;
use swc_common::{sync::Lrc, FileName, SourceMap};
use swc_ecma_ast::*;
use swc_ecma_codegen::{text_writer::JsWriter, Config, Emitter};
use swc_ecma_parser::{lexer::Lexer, Parser, StringInput, Syntax};

use super::n;
use super::setup::SETUP_CODE;
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

        // Keep expression statements and declarations
        match stmt {
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
        "{}\n{}\n_result.n = {};\n_result.sig = {};",
        SETUP_CODE, module_code, n_func, sig_func
    );

    Ok(result)
}

fn extract_main_block(module: &Module) -> Result<BlockStmt, String> {
    match module.body.len() {
        1 => {
            // Pattern: (function() { ... }).call(this)
            let item = &module.body[0];
            if let ModuleItem::Stmt(Stmt::Expr(expr_stmt)) = item {
                if let Expr::Call(call_expr) = &*expr_stmt.expr {
                    if let Callee::Expr(callee) = &call_expr.callee {
                        if let Expr::Member(member) = &**callee {
                            if let Expr::Fn(fn_expr) = &*member.obj {
                                if let Some(body) = &fn_expr.function.body {
                                    return Ok(body.clone());
                                }
                            }
                            // Also try Paren wrapped function
                            if let Expr::Paren(paren) = &*member.obj {
                                if let Expr::Fn(fn_expr) = &*paren.expr {
                                    if let Some(body) = &fn_expr.function.body {
                                        return Ok(body.clone());
                                    }
                                }
                            }
                        }
                    }
                }
            }
            Err("unexpected structure (single item)".into())
        }
        2 => {
            // Pattern 1: var _yt_player={}; (function(g){...}).call(this)
            // Pattern 2: 'use strict'; (function() { ... })()
            let item = &module.body[1];
            if let ModuleItem::Stmt(Stmt::Expr(expr_stmt)) = item {
                if let Expr::Call(call_expr) = &*expr_stmt.expr {
                    if let Callee::Expr(callee) = &call_expr.callee {
                        // Pattern 1: (function(g){...}).call(this) - MemberExpression
                        if let Expr::Member(member) = &**callee {
                            if let Expr::Fn(fn_expr) = &*member.obj {
                                if let Some(body) = &fn_expr.function.body {
                                    let mut block = body.clone();
                                    // Skip `var window = this;`
                                    if !block.stmts.is_empty() {
                                        block.stmts.remove(0);
                                    }
                                    return Ok(block);
                                }
                            }
                            // Also try Paren wrapped function
                            if let Expr::Paren(paren) = &*member.obj {
                                if let Expr::Fn(fn_expr) = &*paren.expr {
                                    if let Some(body) = &fn_expr.function.body {
                                        let mut block = body.clone();
                                        // Skip `var window = this;`
                                        if !block.stmts.is_empty() {
                                            block.stmts.remove(0);
                                        }
                                        return Ok(block);
                                    }
                                }
                            }
                        }
                        // Pattern 2: (function() { ... })() - direct call
                        if let Expr::Fn(fn_expr) = &**callee {
                            if let Some(body) = &fn_expr.function.body {
                                let mut block = body.clone();
                                // Skip `var window = this;`
                                if !block.stmts.is_empty() {
                                    block.stmts.remove(0);
                                }
                                return Ok(block);
                            }
                        }
                        // Also try Paren wrapped function for direct call
                        if let Expr::Paren(paren) = &**callee {
                            if let Expr::Fn(fn_expr) = &*paren.expr {
                                if let Some(body) = &fn_expr.function.body {
                                    let mut block = body.clone();
                                    // Skip `var window = this;`
                                    if !block.stmts.is_empty() {
                                        block.stmts.remove(0);
                                    }
                                    return Ok(block);
                                }
                            }
                            // Pattern: ((function(g){...}).call(this)) - Paren wrapping Call with Member
                            if let Expr::Call(inner_call) = &*paren.expr {
                                if let Callee::Expr(inner_callee) = &inner_call.callee {
                                    if let Expr::Member(member) = &**inner_callee {
                                        if let Expr::Fn(fn_expr) = &*member.obj {
                                            if let Some(body) = &fn_expr.function.body {
                                                let mut block = body.clone();
                                                // Skip `var window = this;`
                                                if !block.stmts.is_empty() {
                                                    block.stmts.remove(0);
                                                }
                                                return Ok(block);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            Err("unexpected structure (two items)".into())
        }
        _ => Err(format!("unexpected structure: {} items", module.body.len())),
    }
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
