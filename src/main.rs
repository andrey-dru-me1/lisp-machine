mod ast;
mod cli;
mod codegen;
mod parser;
mod typechecker;
mod types;

use clap::Parser;
use cli::{Cli, Commands};
use std::fs;
use typechecker::{typecheck_toplevels, TypeEnv};
use types::Type;

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Run { path } => {
            let source = fs::read_to_string(path).expect("Failed to read file");
            compile_and_run(&source, true);
        }
        Commands::Compile { path } => {
            let source = fs::read_to_string(path).expect("Failed to read file");
            compile_and_run(&source, false);
        }
    }
}

fn compile_and_run(source: &str, run: bool) {
    let toplevels = match parser::parse(source) {
        Ok(toplevels) => toplevels,
        Err(e) => {
            eprintln!("Parsing error: {}", e);
            return;
        }
    };

    let mut env: TypeEnv = TypeEnv::new();
    // Arithmetic
    env.insert(
        "+".to_string(),
        Type::Function(vec![Type::Int, Type::Int], Box::new(Type::Int)),
    );
    env.insert(
        "-".to_string(),
        Type::Function(vec![Type::Int, Type::Int], Box::new(Type::Int)),
    );
    env.insert(
        "*".to_string(),
        Type::Function(vec![Type::Int, Type::Int], Box::new(Type::Int)),
    );
    env.insert(
        "/".to_string(),
        Type::Function(vec![Type::Int, Type::Int], Box::new(Type::Int)),
    );

    // Comparison
    env.insert(
        "=".to_string(),
        Type::Function(vec![Type::Int, Type::Int], Box::new(Type::Bool)),
    );
    env.insert(
        "<".to_string(),
        Type::Function(vec![Type::Int, Type::Int], Box::new(Type::Bool)),
    );
    env.insert(
        ">".to_string(),
        Type::Function(vec![Type::Int, Type::Int], Box::new(Type::Bool)),
    );

    if let Err(e) = typecheck_toplevels(&toplevels, &mut env) {
        eprintln!("Type error: {}", e);
        return;
    }

    let full_rust_program = match codegen::codegen_toplevels(&toplevels, &env) {
        Ok(code) => code,
        Err(e) => {
            eprintln!("Codegen error: {}", e);
            return;
        }
    };

    if !run {
        println!("{}", full_rust_program);
        return;
    }

    if let Err(e) = fs::write("output.rs", &full_rust_program) {
        eprintln!("Error writing to file: {}", e);
        return;
    }

    let output = std::process::Command::new("rustc")
        .arg("output.rs")
        .output()
        .expect("Failed to compile generated code");

    if !output.status.success() {
        eprintln!(
            "Compilation failed:\n{}",
            String::from_utf8_lossy(&output.stderr)
        );
        return;
    }

    let run_output = std::process::Command::new("./output")
        .output()
        .expect("Failed to run compiled code");

    print!("{}", String::from_utf8_lossy(&run_output.stdout));

    let _ = fs::remove_file("output.rs");
    let _ = fs::remove_file("output");
}
