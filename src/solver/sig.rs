use swc_ecma_ast::*;

/// Extract signature decryption function from AST node
pub fn extract(stmt: &Stmt) -> Option<String> {
    // Try to match function patterns
    let (block, _name) = match stmt {
        // Pattern: function name(a, b, c) { ... }
        Stmt::Decl(Decl::Fn(fn_decl)) => {
            if fn_decl.function.params.len() != 3 {
                return None;
            }
            (fn_decl.function.body.as_ref()?, Some(&fn_decl.ident.sym))
        }
        // Pattern: var name = function(a, b, c) { ... }
        Stmt::Decl(Decl::Var(var_decl)) => {
            let mut found = None;
            for decl in &var_decl.decls {
                if let Some(init) = &decl.init
                    && let Expr::Fn(fn_expr) = &**init
                        && fn_expr.function.params.len() == 3 {
                            let name = match &decl.name {
                                Pat::Ident(ident) => Some(&ident.id.sym),
                                _ => None,
                            };
                            found = Some((fn_expr.function.body.as_ref()?, name));
                            break;
                        }
            }
            found?
        }
        // Pattern: name = function(a, b, c) { ... }
        Stmt::Expr(expr_stmt) => {
            if let Expr::Assign(assign) = &*expr_stmt.expr {
                if let AssignTarget::Simple(SimpleAssignTarget::Ident(ident)) = &assign.left {
                    if let Expr::Fn(fn_expr) = &*assign.right {
                        if fn_expr.function.params.len() == 3 {
                            (fn_expr.function.body.as_ref()?, Some(&ident.id.sym))
                        } else {
                            return None;
                        }
                    } else {
                        return None;
                    }
                } else {
                    return None;
                }
            } else {
                return None;
            }
        }
        _ => return None,
    };

    // Check the second-to-last statement for the signature pattern
    let stmts = &block.stmts;
    if stmts.len() < 2 {
        return None;
    }

    let relevant_stmt = &stmts[stmts.len() - 2];

    // Match: identifier && (identifier = funcCall(...), ...)
    let Stmt::Expr(expr_stmt) = relevant_stmt else {
        return None;
    };

    let Expr::Bin(bin_expr) = &*expr_stmt.expr else {
        return None;
    };

    if bin_expr.op != BinaryOp::LogicalAnd {
        return None;
    }

    // Handle Paren wrapper: right could be Paren(Seq(...)) or Seq(...)
    let right_expr = match &*bin_expr.right {
        Expr::Paren(paren) => &*paren.expr,
        other => other,
    };

    let Expr::Seq(seq_expr) = right_expr else {
        return None;
    };

    if seq_expr.exprs.is_empty() {
        return None;
    }

    let Expr::Assign(assign_expr) = &*seq_expr.exprs[0] else {
        return None;
    };

    let Expr::Call(call_expr) = &*assign_expr.right else {
        return None;
    };

    // Check for decodeURIComponent pattern in arguments
    let has_decode_uri = call_expr.args.iter().any(|arg| {
        if let Expr::Call(inner_call) = &*arg.expr
            && let Callee::Expr(callee_expr) = &inner_call.callee
                && let Expr::Ident(ident) = &**callee_expr {
                    return &*ident.sym == "decodeURIComponent";
                }
        false
    });

    if !has_decode_uri {
        return None;
    }

    // Extract the function name being called
    let Callee::Expr(callee_expr) = &call_expr.callee else {
        return None;
    };

    let Expr::Ident(func_ident) = &**callee_expr else {
        return None;
    };

    let func_name = &*func_ident.sym;

    // Generate arrow function based on argument count
    if call_expr.args.len() == 1 {
        Some(format!("(sig) => {}(sig)", func_name))
    } else if call_expr.args.len() >= 2 {
        // Get the first argument (usually a literal)
        let first_arg = generate_expr(&call_expr.args[0].expr);
        Some(format!("(sig) => {}({}, sig)", func_name, first_arg))
    } else {
        None
    }
}

fn generate_expr(expr: &Expr) -> String {
    match expr {
        Expr::Lit(Lit::Num(n)) => n.value.to_string(),
        Expr::Lit(Lit::Str(s)) => format!("\"{:?}\"", s.value.as_str()),
        Expr::Ident(ident) => ident.sym.to_string(),
        _ => "null".to_string(),
    }
}
