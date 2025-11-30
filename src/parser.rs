use crate::ast::{Expr, TopLevel};

/// Parses a string into a vector of top-level forms.
pub fn parse(input: &str) -> Result<Vec<TopLevel>, String> {
    let mut tokens = tokenize(input);
    tokens.reverse();
    let mut top_levels = Vec::new();
    while !tokens.is_empty() {
        top_levels.push(parse_toplevel(&mut tokens)?);
    }
    Ok(top_levels)
}

fn parse_toplevel(tokens: &mut Vec<String>) -> Result<TopLevel, String> {
    let expr = parse_expr(tokens)?;
    if let Expr::List(list) = &expr {
        if let Some(Expr::Symbol(s)) = list.get(0) {
            if s == "defun" {
                // (defun name (args...) body)
                if list.len() != 4 {
                    return Err("Invalid defun form".to_string());
                }
                let name = match &list[1] {
                    Expr::Symbol(s) => s.clone(),
                    _ => return Err("Function name must be a symbol".to_string()),
                };
                let args_list = match &list[2] {
                    Expr::List(list) => list,
                    _ => return Err("Function arguments must be a list of symbols".to_string()),
                };
                let args = args_list
                    .iter()
                    .map(|arg| match arg {
                        Expr::Symbol(s) => Ok(s.clone()),
                        _ => Err("Function arguments must be symbols".to_string()),
                    })
                    .collect::<Result<Vec<String>, String>>()?;
                let body = list[3].clone();
                return Ok(TopLevel::Defun(name, args, body));
            }
        }
    }
    Ok(TopLevel::Expr(expr))
}

/// Splits the input string into a vector of tokens.
fn tokenize(input: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut chars = input.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '(' | ')' => tokens.push(c.to_string()),
            '"' => {
                let mut s = String::new();
                s.push(c);
                while let Some(next_c) = chars.next() {
                    s.push(next_c);
                    if next_c == '"' {
                        break;
                    }
                }
                tokens.push(s);
            }
            c if c.is_whitespace() => (),
            _ => {
                let mut s = String::new();
                s.push(c);
                while let Some(&next_c) = chars.peek() {
                    if next_c.is_whitespace() || next_c == '(' || next_c == ')' {
                        break;
                    }
                    s.push(chars.next().unwrap());
                }
                tokens.push(s);
            }
        }
    }
    tokens
}

/// Parses a sequence of tokens into an `Expr`.
fn parse_expr(tokens: &mut Vec<String>) -> Result<Expr, String> {
    let token = tokens.pop().ok_or("Unexpected end of input")?;
    match token.as_str() {
        "(" => {
            let mut list = Vec::new();
            while !tokens.is_empty() && tokens.last().unwrap() != ")" {
                list.push(parse_expr(tokens)?);
            }
            if tokens.pop().is_none() {
                return Err("Missing ')'".to_string());
            }
            Ok(Expr::List(list))
        }
        ")" => Err("Unexpected ')'".to_string()),
        _ => Ok(parse_atom(&token)),
    }
}

/// Parses an atomic token (a number, boolean, string, or symbol).
fn parse_atom(token: &str) -> Expr {
    if token.starts_with('"') && token.ends_with('"') {
        return Expr::String(token[1..token.len() - 1].to_string());
    }
    match token {
        "#t" => Expr::Bool(true),
        "#f" => Expr::Bool(false),
        _ => match token.parse::<i64>() {
            Ok(n) => Expr::Number(n),
            Err(_) => Expr::Symbol(token.to_string()),
        },
    }
}
