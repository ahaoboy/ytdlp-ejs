//! Shared Extraction Logic
//!
//! This module implements the core pattern matching and solver generation,
//! following the same logic as the JS `nsig.ts` and `solvers.ts`.

use crate::provider::JsChallengeError;
use swc_common::{sync::Lrc, FileName, SourceMap, SyntaxContext, DUMMY_SP};
use swc_ecma_ast::*;
use swc_ecma_codegen::{text_writer::JsWriter, Config, Emitter};
use swc_ecma_parser::{lexer::Lexer, Parser, StringInput, Syntax};

/// Information about a function found in the AST
pub struct FunctionInfo {
    /// The name as a JS expression code string (e.g., "funcName" or "obj.prop")
    pub name_expr: String,
    /// The function body statements
    pub body: Vec<Stmt>,
}

/// Extract ALL function infos from a statement.
/// Matches patterns: FunctionDeclaration (non-async), ExpressionStatement(AssignmentExpression, non-async),
/// and VariableDeclaration (returns ALL matching declarators, non-async).
/// This mirrors the JS `identifier` pattern in nsig.ts.
fn extract_function_infos(stmt: &Stmt) -> Vec<FunctionInfo> {
    match stmt {
        // Pattern: function name(...) { ... } — must be non-async
        Stmt::Decl(Decl::Fn(fn_decl)) => {
            if fn_decl.function.is_async || fn_decl.function.body.is_none() {
                return vec![];
            }
            let body = fn_decl.function.body.as_ref().unwrap().stmts.clone();
            vec![FunctionInfo {
                name_expr: fn_decl.ident.sym.to_string(),
                body,
            }]
        }
        // Pattern: name = function(...) { ... } — must be non-async
        Stmt::Expr(expr_stmt) => {
            if let Expr::Assign(assign) = &*expr_stmt.expr {
                if assign.op != AssignOp::Assign {
                    return vec![];
                }
                if let Expr::Fn(fn_expr) = &*assign.right {
                    if fn_expr.function.is_async || fn_expr.function.body.is_none() {
                        return vec![];
                    }
                    let name_expr = expr_to_code_string(&gen_assign_target_to_expr(&assign.left));
                    let body = fn_expr.function.body.as_ref().unwrap().stmts.clone();
                    return vec![FunctionInfo { name_expr, body }];
                }
            }
            vec![]
        }
        // Pattern: var/let/const name = function(...) { ... } — must be non-async
        // CRITICAL: check ALL declarators (JS uses anykey), not just the first
        Stmt::Decl(Decl::Var(var_decl)) => {
            let mut results = Vec::new();
            for decl in &var_decl.decls {
                if let Some(init) = &decl.init
                    && let Expr::Fn(fn_expr) = &**init
                {
                    if fn_expr.function.is_async || fn_expr.function.body.is_none() {
                        continue;
                    }
                    let name_expr = pat_to_code_string(&decl.name);
                    let body = fn_expr.function.body.as_ref().unwrap().stmts.clone();
                    results.push(FunctionInfo { name_expr, body });
                }
            }
            results
        }
        _ => vec![],
    }
}

/// Convert an AssignTarget to an Expr for code generation
fn gen_assign_target_to_expr(target: &AssignTarget) -> Expr {
    match target {
        AssignTarget::Simple(simple) => match simple {
            SimpleAssignTarget::Ident(binding) => Expr::Ident(binding.id.clone()),
            SimpleAssignTarget::Member(member) => Expr::Member(member.clone()),
            _ => Expr::Ident(Ident::new(
                "undefined".into(),
                Default::default(),
                SyntaxContext::empty(),
            )),
        },
        AssignTarget::Pat(assign_pat) => assign_target_pat_to_expr(assign_pat.clone()),
    }
}

/// Convert an AssignTargetPat to an Expr for code generation
fn assign_target_pat_to_expr(pat: AssignTargetPat) -> Expr {
    match pat {
        AssignTargetPat::Array(_arr) => {
            // For array destructuring, just return undefined as placeholder
            Expr::Ident(Ident::new(
                "undefined".into(),
                Default::default(),
                SyntaxContext::empty(),
            ))
        }
        AssignTargetPat::Object(_obj) => Expr::Ident(Ident::new(
            "undefined".into(),
            Default::default(),
            SyntaxContext::empty(),
        )),
        AssignTargetPat::Invalid(_) => Expr::Ident(Ident::new(
            "undefined".into(),
            Default::default(),
            SyntaxContext::empty(),
        )),
    }
}

/// Generate JS code string from an expression
fn expr_to_code_string(expr: &Expr) -> String {
    match expr {
        Expr::Ident(ident) => ident.sym.to_string(),
        Expr::Member(member) => {
            let obj = expr_to_code_string(&member.obj);
            match &member.prop {
                MemberProp::Ident(ident_name) => format!("{}.{}", obj, ident_name.sym),
                MemberProp::Computed(computed) => {
                    let prop = expr_to_code_string(&computed.expr);
                    format!("{}[{}]", obj, prop)
                }
                MemberProp::PrivateName(pn) => format!("{}.#{}", obj, pn.name),
            }
        }
        Expr::Lit(Lit::Str(s)) => {
            // Use JSON-style quoting with proper escaping
            match s.value.as_str() {
                Some(v) => format!("{:?}", v),
                None => "\"\"".to_string(),
            }
        }
        Expr::Lit(Lit::Num(n)) => n.value.to_string(),
        _ => "undefined".to_string(),
    }
}

/// Convert a Pat to a code string
fn pat_to_code_string(pat: &Pat) -> String {
    match pat {
        Pat::Ident(ident) => ident.id.sym.to_string(),
        Pat::Expr(expr) => expr_to_code_string(expr),
        _ => "undefined".to_string(),
    }
}

/// Check if a statement (or any statement in a list) contains the "alr"/"yes" call pattern.
/// This matches: `X.something("alr", "yes")` where something is any identifier.
/// Uses iterative traversal to avoid stack overflow on deeply nested ASTs.
fn has_alr_yes_pattern(stmts: &[Stmt]) -> bool {
    let mut stack: Vec<&Stmt> = stmts.iter().collect();
    let mut expr_stack: Vec<&Expr> = Vec::new();

    while !stack.is_empty() || !expr_stack.is_empty() {
        // Process expressions first
        while let Some(expr) = expr_stack.pop() {
            push_expr_children(expr, &mut expr_stack);
            if is_alr_yes_call(expr) {
                return true;
            }
        }

        // Process next statement
        if let Some(stmt) = stack.pop() {
            match stmt {
                Stmt::Expr(e) => expr_stack.push(&e.expr),
                Stmt::If(s) => {
                    expr_stack.push(&s.test);
                    stack.push(&s.cons);
                    if let Some(alt) = &s.alt {
                        stack.push(alt);
                    }
                }
                Stmt::For(s) => {
                    if let Some(init) = &s.init {
                        match init {
                            VarDeclOrExpr::Expr(e) => expr_stack.push(e),
                            VarDeclOrExpr::VarDecl(_) => {}
                        }
                    }
                    if let Some(test) = &s.test {
                        expr_stack.push(test);
                    }
                    if let Some(update) = &s.update {
                        expr_stack.push(update);
                    }
                    stack.push(&s.body);
                }
                Stmt::While(s) => {
                    expr_stack.push(&s.test);
                    stack.push(&s.body);
                }
                Stmt::DoWhile(s) => {
                    expr_stack.push(&s.test);
                    stack.push(&s.body);
                }
                Stmt::Switch(s) => {
                    expr_stack.push(&s.discriminant);
                    for case in s.cases.iter().rev() {
                        if let Some(test) = &case.test {
                            expr_stack.push(test);
                        }
                        for st in case.cons.iter().rev() {
                            stack.push(st);
                        }
                    }
                }
                Stmt::Block(s) => {
                    for st in s.stmts.iter().rev() {
                        stack.push(st);
                    }
                }
                Stmt::Return(s) => {
                    if let Some(arg) = &s.arg {
                        expr_stack.push(arg);
                    }
                }
                Stmt::Decl(Decl::Var(vd)) => {
                    for d in &vd.decls {
                        if let Some(init) = &d.init {
                            expr_stack.push(init);
                        }
                    }
                }
                _ => {}
            }
        }
    }
    false
}

/// Push children of an expression onto the stack for traversal
fn push_expr_children<'a>(expr: &'a Expr, stack: &mut Vec<&'a Expr>) {
    match expr {
        Expr::Call(c) => {
            for arg in &c.args {
                stack.push(&arg.expr);
            }
            if let Callee::Expr(e) = &c.callee {
                stack.push(e);
            }
        }
        Expr::Paren(p) => stack.push(&p.expr),
        Expr::Seq(s) => {
            for e in s.exprs.iter().rev() {
                stack.push(e);
            }
        }
        Expr::Cond(c) => {
            stack.push(&c.alt);
            stack.push(&c.cons);
            stack.push(&c.test);
        }
        Expr::Bin(b) => {
            stack.push(&b.right);
            stack.push(&b.left);
        }
        Expr::Unary(u) => stack.push(&u.arg),
        _ => {}
    }
}

/// Check if an expression is the "alr"/"yes" call pattern: X.anything("alr", "yes")
fn is_alr_yes_call(expr: &Expr) -> bool {
    if let Expr::Call(call_expr) = expr
        && let Callee::Expr(callee) = &call_expr.callee
            && let Expr::Member(member) = &**callee
                && matches!(&member.prop, MemberProp::Ident(_))
                    && call_expr.args.len() >= 2
                    && is_str_literal(&call_expr.args[0].expr, "alr")
                    && is_str_literal(&call_expr.args[1].expr, "yes")
                {
                    return true;
                }
    false
}

/// Check if an expression is a string literal with the given value
fn is_str_literal(expr: &Expr, value: &str) -> bool {
    matches!(expr, Expr::Lit(Lit::Str(s)) if s.value.as_str() == Some(value))
}

/// Extract solver infos from a statement: find ALL functions containing "alr"/"yes" pattern.
/// Returns all matching function infos (for VarDecl, there may be multiple).
pub fn extract_solver_infos(stmt: &Stmt) -> Vec<FunctionInfo> {
    let func_infos = extract_function_infos(stmt);
    func_infos
        .into_iter()
        .filter(|info| has_alr_yes_pattern(&info.body))
        .collect()
}

/// Helper to parse a JS statement block from a string template
pub fn parse_script(code: &str) -> Result<Vec<Stmt>, JsChallengeError> {
    let cm: Lrc<SourceMap> = Default::default();
    let fm = cm.new_source_file(
        FileName::Custom("script.js".into()).into(),
        code.to_string(),
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

    let stmts = module
        .body
        .into_iter()
        .filter_map(|item| {
            if let ModuleItem::Stmt(stmt) = item {
                Some(stmt)
            } else {
                None
            }
        })
        .collect();
    Ok(stmts)
}

/// Helper to parse a JS expression from a string template
pub fn parse_expr(code: &str) -> Result<Box<Expr>, JsChallengeError> {
    let cm: Lrc<SourceMap> = Default::default();
    let fm = cm.new_source_file(
        FileName::Custom("expr.js".into()).into(),
        format!("({});", code),
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

    if let Some(ModuleItem::Stmt(Stmt::Expr(expr_stmt))) = module.body.first() {
        Ok(expr_stmt.expr.clone())
    } else {
        Err(JsChallengeError::Parse("Failed to parse expression".into()))
    }
}

/// Generate JS code string from an expression using SWC Emitter
pub fn expr_to_code_string_via_codegen(expr: &Expr) -> Result<String, JsChallengeError> {
    let cm: Lrc<SourceMap> = Default::default();
    let mut buf = vec![];
    {
        let writer = JsWriter::new(cm.clone(), "\n", &mut buf, None);
        let mut emitter = Emitter {
            cfg: Config::default(),
            cm: cm.clone(),
            comments: None,
            wr: writer,
        };
        let module = Module {
            span: DUMMY_SP,
            body: vec![ModuleItem::Stmt(Stmt::Expr(ExprStmt {
                span: DUMMY_SP,
                expr: Box::new(expr.clone()),
            }))],
            shebang: None,
        };
        emitter.emit_module(&module).map_err(|e| {
            JsChallengeError::Runtime(format!("Expr code generation failed: {}", e))
        })?;
    }
    let code = String::from_utf8(buf)
        .map_err(|e| JsChallengeError::Runtime(format!("UTF-8 error: {}", e)))?;
    let trimmed = code.trim().trim_end_matches(';');
    Ok(trimmed.to_string())
}

/// Generate the URL-based solver function AST expression that wraps the extracted function.
/// This is equivalent to JS `createSolver(expression)`.
pub fn generate_solver_expr(func_name: &str) -> Result<Box<Expr>, JsChallengeError> {
    let code = format!(
        r#"({{sig, n}}) => {{
  const url = ({})("https://youtube.com/watch?v=yt-dlp-wins", "s", sig ? encodeURIComponent(sig) : undefined);
  url.set("n", n);
  const proto = Object.getPrototypeOf(url);
  const keys = Object.keys(proto).concat(Object.getOwnPropertyNames(proto));
  for (const key of keys) {{
    if (!["constructor", "set", "get", "clone"].includes(key)) {{
      url[key]();
      break;
    }}
  }}
  const s = url.get("s");
  return {{
    sig: s ? decodeURIComponent(s) : null,
    n: url.get("n") ?? null,
  }};
}}"#,
        func_name
    );
    parse_expr(&code)
}

/// Generate the n solver wrapper expression from the solver expression.
/// Matches JS `makeSolver(result, "n")`.
pub fn generate_n_solver_expr(solver_expr: &Expr) -> Result<Box<Expr>, JsChallengeError> {
    let solver_code = expr_to_code_string_via_codegen(solver_expr)?;
    let code = format!("(n) => ({})({{ n }}).n", solver_code);
    parse_expr(&code)
}

/// Generate the sig solver wrapper expression from the solver expression.
/// Matches JS `makeSolver(result, "sig")`.
pub fn generate_sig_solver_expr(solver_expr: &Expr) -> Result<Box<Expr>, JsChallengeError> {
    let solver_code = expr_to_code_string_via_codegen(solver_expr)?;
    let code = format!("(sig) => ({})({{ sig }}).sig", solver_code);
    parse_expr(&code)
}

/// Generate the multiTry consensus wrapper expression.
/// Equivalent to JS `multiTry(generators)`.
pub fn generate_multi_try_expr(solvers: &[Box<Expr>]) -> Result<Box<Expr>, JsChallengeError> {
    if solvers.is_empty() {
        return parse_expr("(_input) => { throw 'no solutions'; }");
    }
    if solvers.len() == 1 {
        return Ok(solvers[0].clone());
    }
    let mut solver_codes = Vec::new();
    for solver in solvers {
        solver_codes.push(expr_to_code_string_via_codegen(solver)?);
    }
    let generators_code = solver_codes.join(", ");
    let code = format!(
        r#"(_input) => {{
  const _results = new Set();
  const errors = [];
  for (const _generator of [{generators_code}]) {{
    try {{
      _results.add(_generator(_input));
    }} catch (e) {{
      errors.push(e);
    }}
  }}
  if (!_results.size) {{
    throw "no solutions: " + errors.join(", ");
  }}
  if (_results.size !== 1) {{
    throw "invalid solutions: " + [..._results].map(x => JSON.stringify(x)).join(", ");
  }}
  return _results.values().next().value;
}}"#
    );
    parse_expr(&code)
}
