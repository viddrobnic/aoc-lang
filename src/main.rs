use std::{fs, path::PathBuf, process::exit};

use clap::{Parser, Subcommand};
use language_server::Server;

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    Run {
        /// Path of the file to run
        path: PathBuf,
    },
    Lsp {
        /// Optional debug path
        #[arg(short, long)]
        debug_log_path: Option<PathBuf>,
    },
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Commands::Run { path } => run(path),
        Commands::Lsp { debug_log_path } => {
            let mut server = Server::new(debug_log_path);
            server.start()
        }
    }
}

fn run(path: PathBuf) {
    let input = match fs::read_to_string(path) {
        Ok(input) => input,
        Err(err) => {
            println!("Failed to read input file: {err}");
            exit(1);
        }
    };

    let program = match parser::parse(&input) {
        Ok(program) => program,
        Err(err) => {
            println!(
                "Syntax error on line {}, character {}:\n  {}",
                err.range.start.line + 1,
                err.range.start.character + 1,
                err
            );
            exit(1);
        }
    };

    match runtime::run(&program) {
        Ok(_) => (),
        Err(err) => {
            println!(
                "Runtime error on line {}, character {}:\n  {}",
                err.range.start.line + 1,
                err.range.start.character + 1,
                err
            );
            exit(1);
        }
    }
}
