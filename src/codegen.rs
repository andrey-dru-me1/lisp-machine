use crate::ast::{Expr, TopLevel};
use crate::typechecker::TypeEnv;

pub fn codegen_toplevels(toplevels: &[TopLevel], env: &TypeEnv) -> Result<String, String> {
    let mut functions = Vec::new();
    let mut main_exprs = Vec::new();

    for toplevel in toplevels {
        match toplevel {
            TopLevel::Defun(name, args, body) => {
                // For now, assume all args are i64 and return is i64
                let args_str = args
                    .iter()
                    .map(|arg| format!("{}: i64", arg))
                    .collect::<Vec<_>>()
                    .join(", ");
                let body_str = codegen_expr(body, env)?;
                functions.push(format!(
                    "fn {}({}) -> i64 {{ {} }}",
                    name, args_str, body_str
                ));
            }
            TopLevel::Expr(expr) => {
                main_exprs.push(codegen_expr(expr, env)?);
            }
        }
    }

    let main_body = main_exprs.join(";\n    ");
    let main_fn = format!("fn main() {{\n    {};\n}}", main_body);
    functions.push(main_fn);

    Ok(functions.join("\n\n"))
}

/// Generates Rust code from an expression.
fn codegen_expr(expr: &Expr, env: &TypeEnv) -> Result<String, String> {
    match expr {
        Expr::Number(n) => Ok(n.to_string()),
        Expr::Bool(b) => Ok(b.to_string()),
        Expr::String(s) => Ok(format!("\"{}\"", s)),
        Expr::Symbol(s) => Ok(s.clone()),
        Expr::List(list) => {
            if list.is_empty() {
                return Err("Cannot generate code for empty list".to_string());
            }

            let head = &list[0];
            let args = &list[1..];

            if let Expr::Symbol(s) = head {
                match s.as_str() {
                    "print" => {
                        let arg = codegen_expr(&args[0], env)?;
                        return Ok(format!("{{ println!(\"{{}}\", {}); 0 }}", arg));
                    }
                    "if" => {
                        let cond = codegen_expr(&args[0], env)?;
                        let then = codegen_expr(&args[1], env)?;
                        let else_ = codegen_expr(&args[2], env)?;
                        return Ok(format!("if {} {{ {} }} else {{ {} }}", cond, then, else_));
                    }
                    "let" => {
                        let mut statements = Vec::new();
                        let bindings_list = match &args[0] {
                            Expr::List(list) => list,
                            _ => return Err("Invalid let bindings".to_string()),
                        };
                        for binding in bindings_list {
                            if let Expr::List(pair) = binding {
                                if let Expr::Symbol(name) = &pair[0] {
                                    let val = codegen_expr(&pair[1], env)?;
                                    statements.push(format!("let {} = {};", name, val));
                                }
                            }
                        }
                        let body = codegen_expr(&args[1], env)?;
                        statements.push(body);
                        return Ok(format!("{{ {} }}", statements.join("\n")));
                    }
                    _ => {}
                }

                let op = match s.as_str() {
                    "+" | "-" | "*" | "/" | "=" | "<" | ">" => s,
                    _ => "",
                };

                if !op.is_empty() {
                    let rust_op = match op {
                        "=" => "==",
                        _ => op,
                    };
                    if args.len() != 2 {
                        return Err(format!("'{}' takes exactly two arguments", s));
                    }
                    let arg1 = codegen_expr(&args[0], env)?;
                    let arg2 = codegen_expr(&args[1], env)?;
                    return Ok(format!("({} {} {})", arg1, rust_op, arg2));
                }

                if env.contains_key(s) {
                    // It's a function call
                    let args_str = args
                        .iter()
                        .map(|arg| codegen_expr(arg, env))
                        .collect::<Result<Vec<_>, String>>()?
                        .join(", ");
                    return Ok(format!("{}({})", s, args_str));
                }

                Err(format!("Unknown function: {}", s))
            } else {
                Err("First element of list is not a function symbol".to_string())
            }
        }
    }
}
