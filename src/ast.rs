/// Represents a top-level form in our language.
#[derive(Debug, Clone, PartialEq)]
pub enum TopLevel {
    /// A function definition.
    Defun(String, Vec<String>, Expr),
    /// An expression to be evaluated.
    Expr(Expr),
}

/// Represents a single expression in our Lisp-like language.
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    /// A symbol, like `add` or `x`.
    Symbol(String),
    /// A 64-bit integer number.
    Number(i64),
    /// A boolean value (`#t` or `#f`).
    Bool(bool),
    /// A string literal.
    String(String),
    /// A list of expressions, like `(+ 1 2)`.
    List(Vec<Expr>),
}
