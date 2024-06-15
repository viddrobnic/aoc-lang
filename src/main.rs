use std::{fs, path::PathBuf, process::exit};

use clap::Parser;

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Path of the file to run
    path: PathBuf,
}

fn main() {
    let cli = Cli::parse();
    run(cli.path);
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
                err.range.start.line, err.range.start.character, err
            );
            exit(1);
        }
    };

    match runtime::run(&program) {
        Ok(_) => (),
        Err(err) => {
            println!(
                "Runtime error on line {}, character {}:\n  {}",
                err.range.start.line, err.range.start.character, err
            );
            exit(1);
        }
    }
}
