use swc_ecma_ast::*;

/// Pattern matching types for AST structure matching
#[derive(Debug, Clone)]
pub enum Pattern {
    Any,
    Exact(Box<PatternValue>),
    Or(Vec<Pattern>),
    AnyKey(Vec<Pattern>),
}

#[derive(Debug, Clone)]
pub enum PatternValue {
    String(String),
    Number(f64),
    Bool(bool),
    Null,
}

/// Check if a value is one of the given options
pub fn is_one_of<T: PartialEq>(value: &T, options: &[T]) -> bool {
    options.contains(value)
}

/// Helper to check if an expression is an identifier with a specific name
pub fn is_identifier(expr: &Expr, name: &str) -> bool {
    matches!(expr, Expr::Ident(ident) if &*ident.sym == name)
}

/// Helper to get identifier name from expression
pub fn get_ident_name(expr: &Expr) -> Option<&str> {
    match expr {
        Expr::Ident(ident) => Some(&ident.sym),
        _ => None,
    }
}

/// Helper to get identifier name from pattern
pub fn get_pat_ident_name(pat: &Pat) -> Option<&str> {
    match pat {
        Pat::Ident(ident) => Some(&ident.id.sym),
        _ => None,
    }
}
