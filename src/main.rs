//! # run
//!
//! A simple scripting language for CLI automation, inspired by shell scripting and Makefiles.
//! Define functions in a `Runfile` (or `~/.runfile`) and call them from the command line to streamline your development workflow.
//!
//! ## Usage
//!
//! - Run a script file: `run myscript.run`
//! - Call a function: `run build`, `run docker shell app`
//! - Pass arguments: `run start dev`, `run git commit "Initial commit"`
//! - Interactive shell: `run`
//!
//! See README.md for more details and examples.

mod ast;
mod parser;
mod interpreter;

use clap::Parser as ClapParser;
use std::fs;
use std::path::PathBuf;
use std::io::{self, Write};

const VERSION: &str = env!("CARGO_PKG_VERSION");
const VERSION_WITH_V: &str = concat!("v", env!("CARGO_PKG_VERSION"));

/// CLI arguments for the run tool.
#[derive(ClapParser)]
#[command(name = "run")]
#[command(version = VERSION_WITH_V)]
#[command(about = "A simple scripting language for CLI automation", long_about = None)]
struct Cli {
    /// Script file to execute, or function name to call
    #[arg(value_name = "FILE_OR_FUNCTION")]
    first_arg: Option<String>,

    /// Additional arguments for function calls
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    args: Vec<String>,

    /// List all available functions from the Runfile
    #[arg(short, long)]
    list: bool,
}

/// Entry point for the CLI tool.
fn main() {
    let cli = Cli::parse();

    // Handle --list flag
    if cli.list {
        list_functions();
        return;
    }

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

                execute_script(&script, Some(path.to_string_lossy().to_string()));
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

/// List all available functions from the Runfile.
fn list_functions() {
    let config_content = match load_config() {
        Some(content) => content,
        None => {
            eprintln!("Error: No Runfile found. Create ~/.runfile or ./Runfile to define functions.");
            std::process::exit(1);
        }
    };

    // Parse the config to extract function names
    match parser::parse_script(&config_content) {
        Ok(program) => {
            let mut functions = Vec::new();
            for statement in program.statements {
                if let ast::Statement::SimpleFunctionDef { name, .. } = statement {
                    functions.push(name);
                }
            }

            if functions.is_empty() {
                println!("No functions defined in Runfile.");
                // Exit with success since the file was found and parsed correctly
                std::process::exit(0);
            } else {
                println!("Available functions:");
                for func in functions {
                    println!("  {}", func);
                }
            }
        }
        Err(e) => {
            eprintln!("Error parsing Runfile: {}", e);
            std::process::exit(1);
        }
    }
}

/// Parse and execute a script file.
///
/// # Arguments
/// * `script` - The script source code to parse and execute.
/// * `filename` - Optional filename for better error messages.
fn execute_script(script: &str, filename: Option<String>) {
    // Parse the script
    let program = match parser::parse_script(script) {
        Ok(prog) => prog,
        Err(e) => {
            print_parse_error(&e, script, filename.as_deref());
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

/// Print a parse error with context from the source code.
fn print_parse_error(error: &Box<dyn std::error::Error>, source: &str, filename: Option<&str>) {
    let error_str = error.to_string();

    // Try to extract line information from pest error
    if let Some(line_info) = extract_line_from_error(&error_str) {
        let file_prefix = filename.map(|f| format!("{}:", f)).unwrap_or_default();
        eprintln!("Parse error in {}line {}: {}", file_prefix, line_info.line, line_info.message);

        // Show the problematic line if we can extract it
        if let Some(line_content) = get_line(source, line_info.line) {
            eprintln!();
            eprintln!("  {} | {}", line_info.line, line_content);
            eprintln!("  {} | {}", " ".repeat(line_info.line.to_string().len()), "^".repeat(line_content.trim().len().max(1)));
        }
    } else {
        eprintln!("Parse error: {}", error_str);
    }
}

struct LineInfo {
    line: usize,
    message: String,
}

/// Extract line number from pest error message.
fn extract_line_from_error(error_str: &str) -> Option<LineInfo> {
    // Pest errors often contain " --> line:col" or similar patterns
    // This is a simple heuristic parser
    if let Some(pos) = error_str.find(" --> ") {
        let rest = &error_str[pos + 5..];
        if let Some(colon_pos) = rest.find(':') {
            if let Ok(line) = rest[..colon_pos].parse::<usize>() {
                return Some(LineInfo {
                    line,
                    message: error_str.to_string(),
                });
            }
        }
    }
    None
}

/// Get a specific line from source code.
fn get_line(source: &str, line_num: usize) -> Option<String> {
    source.lines().nth(line_num.saturating_sub(1)).map(|s| s.to_string())
}

/// Load function definitions from config and call a function with arguments.
///
/// # Arguments
/// * `function_name` - The function to call (may be nested, e.g. "docker shell").
/// * `args` - Arguments to pass to the function.
fn run_function_call(function_name: &str, args: &[String]) {
    // Load the config file from ~/.runfile or ./Runfile
    let config_content = match load_config() {
        Some(content) => content,
        None => {
            eprintln!("Error: No Runfile found. Create ~/.runfile or ./Runfile to define functions.");
            std::process::exit(1);
        }
    };

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
            print_parse_error(&e, &config_content, Some("Runfile"));
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

/// Get the user's home directory in a cross-platform way.
fn get_home_dir() -> Option<PathBuf> {
    // Try HOME first (Unix-like systems)
    if let Some(home) = std::env::var_os("HOME") {
        return Some(PathBuf::from(home));
    }

    // Try USERPROFILE (Windows)
    if let Some(userprofile) = std::env::var_os("USERPROFILE") {
        return Some(PathBuf::from(userprofile));
    }

    // Try HOMEDRIVE + HOMEPATH (older Windows)
    if let (Some(homedrive), Some(homepath)) = (
        std::env::var_os("HOMEDRIVE"),
        std::env::var_os("HOMEPATH")
    ) {
        let mut path = PathBuf::from(homedrive);
        path.push(homepath);
        return Some(path);
    }

    None
}

/// Search for a Runfile in the current directory or upwards, then fallback to ~/.runfile.
/// Returns Some(content) if a file is found (even if empty), or None if no file exists.
fn load_config() -> Option<String> {
    // Start from the current directory and search upwards
    let mut current_dir = match std::env::current_dir() {
        Ok(dir) => dir,
        Err(_) => {
            // If we can't get current dir, fall back to home directory only
            return load_home_runfile();
        }
    };

    // Get home directory for boundary check
    let home_dir = get_home_dir();

    // Search upwards from current directory
    loop {
        let runfile_path = current_dir.join("Runfile");
        if runfile_path.exists() {
            // File exists, read it (even if empty)
            if let Ok(content) = fs::read_to_string(&runfile_path) {
                return Some(content);
            }
        }

        // Check if we've reached the home directory or root
        let reached_boundary = if let Some(ref home) = home_dir {
            current_dir == *home || current_dir == PathBuf::from("/") || current_dir == PathBuf::from("\\")
        } else {
            current_dir == PathBuf::from("/") || current_dir == PathBuf::from("\\")
        };

        if reached_boundary {
            break;
        }

        // Move up one directory
        match current_dir.parent() {
            Some(parent) => current_dir = parent.to_path_buf(),
            None => break, // Reached root
        }
    }

    // Finally, try ~/.runfile as a fallback
    load_home_runfile()
}

/// Load ~/.runfile from the user's home directory.
/// Returns Some(content) if found, or None otherwise.
fn load_home_runfile() -> Option<String> {
    if let Some(home) = get_home_dir() {
        let runfile_path = home.join(".runfile");
        if runfile_path.exists() {
            if let Ok(content) = fs::read_to_string(runfile_path) {
                return Some(content);
            }
        }
    }
    None
}

/// Start an interactive shell (REPL) for the run scripting language.
fn run_repl() {
    println!("Run Shell v{}", VERSION);
    println!("Type 'exit' or press Ctrl+D to quit\n");

    let mut interpreter = interpreter::Interpreter::new();

    // Load Runfile functions into the REPL
    if let Some(config_content) = load_config() {
        match parser::parse_script(&config_content) {
            Ok(program) => {
                if let Err(e) = interpreter.execute(program) {
                    eprintln!("Warning: Error loading Runfile functions: {}", e);
                }
            }
            Err(e) => {
                eprintln!("Warning: Error parsing Runfile: {}", e);
            }
        }
    }

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
                        print_parse_error(&e, input, None);
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
