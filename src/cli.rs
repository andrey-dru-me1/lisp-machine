use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Compiles and runs a Lisp file
    Run {
        /// The path to the Lisp file to run
        #[arg(short, long)]
        path: String,
    },
    /// Compiles a Lisp file to Rust
    Compile {
        /// The path to the Lisp file to compile
        #[arg(short, long)]
        path: String,
    },
}
