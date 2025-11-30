/// Represents the types in our Lisp-like language.
#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    /// The 64-bit integer type.
    Int,
    /// The boolean type.
    Bool,
    /// The string type.
    String,
    /// A function type, with a list of argument types and a return type.
    Function(Vec<Type>, Box<Type>),
}
