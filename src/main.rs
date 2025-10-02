mod ast;
mod parser;
mod interpreter;

use clap::Parser as ClapParser;
use std::fs;
use std::path::PathBuf;
use std::io::{self, Write};

#[derive(ClapParser)]
#[command(name = "run")]
#[command(about = "A simple scripting language for CLI automation", long_about = None)]
struct Cli {
    /// Script file to execute, or function name to call
    #[arg(value_name = "FILE_OR_FUNCTION")]
    first_arg: Option<String>,

    /// Additional arguments for function calls
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    args: Vec<String>,
}

fn main() {
    let cli = Cli::parse();

    match cli.first_arg {
        Some(first_arg) => {
            // Check if it's a file that exists
            let path = PathBuf::from(&first_arg);
            if path.exists() && path.is_file() {
                // File mode: read and execute script
                let script = match fs::read_to_string(&path) {
                    Ok(content) => content,
                    Err(e) => {
                        eprintln!("Error reading file '{}': {}", path.display(), e);
                        std::process::exit(1);
                    }
                };

                execute_script(&script);
            } else {
                // Function call mode: load config and call function with args
                run_function_call(&first_arg, &cli.args);
            }
        }
        None => {
            // REPL mode: interactive shell
            run_repl();
        }
    }
}

fn execute_script(script: &str) {
    // Parse the script
    let program = match parser::parse_script(script) {
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

fn run_function_call(function_name: &str, args: &[String]) {
    // Load the config file from ~/.runfile or ./Runfile
    let config_content = load_config();

    if config_content.is_empty() {
        eprintln!("Error: No Runfile found. Create ~/.runfile or ./Runfile to define functions.");
        std::process::exit(1);
    }

    // Parse the config to load function definitions
    let mut interpreter = interpreter::Interpreter::new();

    match parser::parse_script(&config_content) {
        Ok(program) => {
            // Execute to load function definitions
            if let Err(e) = interpreter.execute(program) {
                eprintln!("Error loading functions: {}", e);
                std::process::exit(1);
            }
        }
        Err(e) => {
            eprintln!("Error parsing Runfile: {}", e);
            std::process::exit(1);
        }
    }

    // Now execute the function call with arguments
    // For nested commands, try different combinations:
    // e.g., "docker shell app" -> try "docker:shell" with arg "app"
    if let Err(e) = interpreter.call_function_without_parens(function_name, args) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn load_config() -> String {
    // Try local Runfile first
    if let Ok(content) = fs::read_to_string("./Runfile") {
        return content;
    }

    // Try ~/.runfile
    if let Some(home) = std::env::var_os("HOME") {
        let home_path = PathBuf::from(home);
        let runfile_path = home_path.join(".runfile");
        if let Ok(content) = fs::read_to_string(runfile_path) {
            return content;
        }
    }

    String::new()
}

fn run_repl() {
    println!("Run Shell v0.1.0");
    println!("Type 'exit' or press Ctrl+D to quit\n");

    let mut interpreter = interpreter::Interpreter::new();
    let stdin = io::stdin();
    let mut stdout = io::stdout();

    loop {
        // Print prompt
        print!("> ");
        stdout.flush().unwrap();

        // Read line
        let mut input = String::new();
        match stdin.read_line(&mut input) {
            Ok(0) => {
                // EOF (Ctrl+D)
                println!("\nGoodbye!");
                break;
            }
            Ok(_) => {
                let input = input.trim();

                // Check for exit command
                if input == "exit" || input == "quit" {
                    println!("Goodbye!");
                    break;
                }

                // Skip empty lines
                if input.is_empty() {
                    continue;
                }

                // Try to parse and execute the input
                match parser::parse_script(input) {
                    Ok(program) => {
                        if let Err(e) = interpreter.execute(program) {
                            eprintln!("Error: {}", e);
                        }
                    }
                    Err(e) => {
                        eprintln!("Parse error: {}", e);
                    }
                }
            }
            Err(e) => {
                eprintln!("Error reading input: {}", e);
                break;
            }
        }
    }
}
