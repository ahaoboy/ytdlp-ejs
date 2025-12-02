use swc_ecma_ast::*;

/// Extract n parameter decryption function from AST node
pub fn extract(stmt: &Stmt) -> Option<String> {
    // Pattern 1: var xxx = [funcName] or xxx = [funcName]
    if let Some(name) = extract_array_pattern(stmt) {
        return Some(make_solver_func(&name));
    }

    // Pattern 2: Fallback - function with try/catch returning X[12] + Y
    if let Some(name) = extract_try_catch_pattern(stmt) {
        return Some(make_solver_func(&name));
    }

    None
}

fn extract_array_pattern(stmt: &Stmt) -> Option<String> {
    match stmt {
        // var xxx = [funcName] - must be "var" kind only
        Stmt::Decl(Decl::Var(var_decl)) => {
            // Only match "var" declarations, not "let" or "const"
            if var_decl.kind != VarDeclKind::Var {
                return None;
            }
            for decl in &var_decl.decls {
                if let Some(init) = &decl.init {
                    if let Expr::Array(arr) = &**init {
                        if arr.elems.len() == 1 {
                            if let Some(Some(ExprOrSpread { expr, .. })) = arr.elems.first() {
                                if let Expr::Ident(ident) = &**expr {
                                    return Some(ident.sym.to_string());
                                }
                            }
                        }
                    }
                }
            }
            None
        }
        // xxx = [funcName] - left must be Identifier, operator must be "="
        Stmt::Expr(expr_stmt) => {
            if let Expr::Assign(assign) = &*expr_stmt.expr {
                // Check left is Identifier and operator is "="
                if let AssignTarget::Simple(SimpleAssignTarget::Ident(_)) = &assign.left {
                    if assign.op == AssignOp::Assign {
                        if let Expr::Array(arr) = &*assign.right {
                            if arr.elems.len() == 1 {
                                if let Some(Some(ExprOrSpread { expr, .. })) = arr.elems.first() {
                                    if let Expr::Ident(ident) = &**expr {
                                        return Some(ident.sym.to_string());
                                    }
                                }
                            }
                        }
                    }
                }
            }
            None
        }
        _ => None,
    }
}

fn extract_try_catch_pattern(stmt: &Stmt) -> Option<String> {
    let (block, name) = match stmt {
        // name = function(a) { ... }
        Stmt::Expr(expr_stmt) => {
            if let Expr::Assign(assign) = &*expr_stmt.expr {
                if let AssignTarget::Simple(SimpleAssignTarget::Ident(ident)) = &assign.left {
                    if let Expr::Fn(fn_expr) = &*assign.right {
                        if fn_expr.function.params.len() == 1 {
                            (fn_expr.function.body.as_ref()?, ident.id.sym.to_string())
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
        // function name(a) { ... }
        Stmt::Decl(Decl::Fn(fn_decl)) => {
            if fn_decl.function.params.len() == 1 {
                (
                    fn_decl.function.body.as_ref()?,
                    fn_decl.ident.sym.to_string(),
                )
            } else {
                return None;
            }
        }
        _ => return None,
    };

    // Check second-to-last statement for try/catch pattern
    let stmts = &block.stmts;
    if stmts.len() < 2 {
        return None;
    }

    let try_stmt = &stmts[stmts.len() - 2];
    let Stmt::Try(try_block) = try_stmt else {
        return None;
    };

    let Some(catch_clause) = &try_block.handler else {
        return None;
    };

    // Check catch block for: return X[12] + Y
    let catch_body = &catch_clause.body.stmts;
    if catch_body.len() != 1 {
        return None;
    }

    let Stmt::Return(return_stmt) = &catch_body[0] else {
        return None;
    };

    let Some(arg) = &return_stmt.arg else {
        return None;
    };

    // Match: X[literal] + Y
    let Expr::Bin(bin_expr) = &**arg else {
        return None;
    };

    if bin_expr.op != BinaryOp::Add {
        return None;
    }

    let Expr::Member(member_expr) = &*bin_expr.left else {
        return None;
    };

    if !member_expr.prop.is_computed() {
        return None;
    }

    // Verify it's accessing with a literal index
    if let MemberProp::Computed(computed) = &member_expr.prop {
        if !matches!(&*computed.expr, Expr::Lit(Lit::Num(_))) {
            return None;
        }
    }

    Some(name)
}

fn make_solver_func(name: &str) -> String {
    format!("(n) => {}(n)", name)
}
