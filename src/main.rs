mod ast;
mod parser;
mod interpreter;

use clap::Parser as ClapParser;
use std::fs;
use std::path::PathBuf;

#[derive(ClapParser)]
#[command(name = "run")]
#[command(about = "A simple scripting language for CLI automation", long_about = None)]
struct Cli {
    /// Script file to execute
    #[arg(value_name = "FILE")]
    file: PathBuf,
}

fn main() {
    let cli = Cli::parse();

    // Read the script file
    let script = match fs::read_to_string(&cli.file) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Error reading file '{}': {}", cli.file.display(), e);
            std::process::exit(1);
        }
    };

    // Parse the script
    let program = match parser::parse_script(&script) {
        Ok(prog) => prog,
        Err(e) => {
            eprintln!("Parse error: {}", e);
            std::process::exit(1);
        }
    };

    // Execute the program
    let mut interpreter = interpreter::Interpreter::new();
    if let Err(e) = interpreter.execute(program) {
        eprintln!("Execution error: {}", e);
        std::process::exit(1);
    }
}
