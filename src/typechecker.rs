use crate::ast::{Expr, TopLevel};
use crate::types::Type;
use std::collections::HashMap;

/// A type checking environment, which stores the types of variables.
pub type TypeEnv = HashMap<String, Type>;

pub fn typecheck_toplevels(toplevels: &[TopLevel], env: &mut TypeEnv) -> Result<Type, String> {
    // First pass: add function signatures to the environment
    for toplevel in toplevels {
        if let TopLevel::Defun(name, args, _body) = toplevel {
            // For now, assume all args are Int and return type will be inferred.
            // This is a simplification. A real implementation would need annotations
            // or a more complex inference algorithm (like Hindley-Milner).
            let arg_types = vec![Type::Int; args.len()];

            // Temporarily add function to env to allow recursion, with a dummy return type
            let temp_func_type = Type::Function(arg_types.clone(), Box::new(Type::Int)); // Dummy return
            env.insert(name.clone(), temp_func_type);
        }
    }

    // Second pass: type-check function bodies and expressions
    let mut last_expr_type = Ok(Type::Int);
    for toplevel in toplevels {
        match toplevel {
            TopLevel::Defun(name, args, body) => {
                let arg_types = vec![Type::Int; args.len()];
                let mut new_env = env.clone();
                for (arg_name, arg_type) in args.iter().zip(arg_types.iter()) {
                    new_env.insert(arg_name.clone(), arg_type.clone());
                }

                let return_type = typecheck_expr(body, &new_env)?;
                let func_type = Type::Function(arg_types, Box::new(return_type));
                env.insert(name.clone(), func_type); // Update with the real return type
            }
            TopLevel::Expr(expr) => {
                last_expr_type = typecheck_expr(expr, env);
            }
        }
    }

    last_expr_type
}

/// Type-checks an expression in a given environment.
fn typecheck_expr(expr: &Expr, env: &TypeEnv) -> Result<Type, String> {
    match expr {
        Expr::Number(_) => Ok(Type::Int),
        Expr::Bool(_) => Ok(Type::Bool),
        Expr::String(_) => Ok(Type::String),
        Expr::Symbol(s) => env
            .get(s)
            .cloned()
            .ok_or(format!("Unbound variable: {}", s)),
        Expr::List(list) => {
            if list.is_empty() {
                return Err("Cannot typecheck empty list".to_string());
            }

            let head = &list[0];
            let args = &list[1..];

            if let Expr::Symbol(s) = head {
                match s.as_str() {
                    "print" => {
                        if args.len() != 1 {
                            return Err("'print' takes exactly one argument".to_string());
                        }
                        // Typecheck the argument, but we don't care about its type
                        // as long as it's printable. For now, we allow anything.
                        let _ = typecheck_expr(&args[0], env)?;
                        return Ok(Type::Int); // print returns 0
                    }
                    "if" => {
                        if args.len() != 3 {
                            return Err("'if' requires a condition, a then-branch, and an else-branch".to_string());
                        }
                        let cond_type = typecheck_expr(&args[0], env)?;
                        if cond_type != Type::Bool {
                            return Err("Condition in 'if' must be a boolean".to_string());
                        }
                        let then_type = typecheck_expr(&args[1], env)?;
                        let else_type = typecheck_expr(&args[2], env)?;
                        if then_type != else_type {
                            return Err("Branches of 'if' must have the same type".to_string());
                        }
                        return Ok(then_type);
                    }
                    "let" => {
                        if args.len() != 2 {
                            return Err("'let' requires a list of bindings and a body".to_string());
                        }
                        let bindings_list = match &args[0] {
                            Expr::List(list) => list,
                            _ => return Err("First argument to 'let' must be a list of bindings".to_string()),
                        };

                        let mut new_env = env.clone();
                        for binding in bindings_list {
                            if let Expr::List(pair) = binding {
                                if pair.len() != 2 {
                                    return Err("Each binding in 'let' must be a pair of (symbol value)".to_string());
                                }
                                if let Expr::Symbol(name) = &pair[0] {
                                    let val_type = typecheck_expr(&pair[1], &new_env)?;
                                    new_env.insert(name.clone(), val_type);
                                } else {
                                    return Err("First element of a binding pair must be a symbol".to_string());
                                }
                            } else {
                                return Err("Each binding in 'let' must be a list".to_string());
                            }
                        }

                        return typecheck_expr(&args[1], &new_env);
                    }
                    _ => {}
                }
            }

            let func_type = typecheck_expr(head, env)?;

            if let Type::Function(param_types, return_type) = func_type {
                let arg_exprs = args;
                if arg_exprs.len() != param_types.len() {
                    return Err("Incorrect number of arguments".to_string());
                }

                for (arg_expr, param_type) in arg_exprs.iter().zip(param_types.iter()) {
                    let arg_type = typecheck_expr(arg_expr, env)?;
                    if &arg_type != param_type {
                        return Err(format!("Type mismatch: expected {:?}, got {:?}", param_type, arg_type));
                    }
                }

                Ok(*return_type)
            } else {
                Err("First element of list is not a function".to_string())
            }
        }
    }
}
