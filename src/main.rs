use std::{fs, path::PathBuf};

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
        Err(err) => panic!("Failed to read input file: {err}"),
    };

    // TODO: Better error handling
    let program = parser::parse(&input).unwrap();
    runtime::run(&program).unwrap();
}
