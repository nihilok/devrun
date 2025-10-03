// Interpreter to execute the AST

use crate::ast::{Expression, Program, Statement};
use std::collections::HashMap;
use std::process::{Command, Stdio};

pub struct Interpreter {
    variables: HashMap<String, String>,
    functions: HashMap<String, Vec<Statement>>,
    simple_functions: HashMap<String, String>,
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
            functions: HashMap::new(),
            simple_functions: HashMap::new(),
        }
    }

    pub fn execute(&mut self, program: Program) -> Result<(), Box<dyn std::error::Error>> {
        for statement in program.statements {
            self.execute_statement(statement)?;
        }
        Ok(())
    }

    pub fn call_function_without_parens(
        &mut self,
        function_name: &str,
        args: &[String],
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Strategy: try to match function names in different ways
        // 1. Direct match: "docker_shell" with args
        // 2. If args exist, try first arg as subcommand: "docker" + "shell" -> "docker:shell"
        // 3. Try replacing underscores with colons: "docker_shell" -> "docker:shell"

        // Try direct match first
        if let Some(command_template) = self.simple_functions.get(function_name) {
            let command = self.substitute_args(command_template, args);
            return self.execute_command(&command);
        }

        // If we have args, try treating the first arg as a subcommand
        if !args.is_empty() {
            let nested_name = format!("{}:{}", function_name, args[0]);
            if let Some(command_template) = self.simple_functions.get(&nested_name) {
                let command = self.substitute_args(command_template, &args[1..]);
                return self.execute_command(&command);
            }
        }

        // Try replacing underscores with colons
        let with_colons = function_name.replace("_", ":");
        if with_colons != function_name
            && let Some(command_template) = self.simple_functions.get(&with_colons)
        {
            let command = self.substitute_args(command_template, args);
            return self.execute_command(&command);
        }

        // Check for full function definitions
        if let Some(body) = self.functions.get(function_name).cloned() {
            for stmt in body {
                self.execute_statement(stmt)?;
            }
            return Ok(());
        }

        Err(format!("Function '{}' not found", function_name).into())
    }

    pub fn call_function_with_args(
        &mut self,
        function_name: &str,
        args: &[String],
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Direct function call with args in parentheses
        // Try to find the function and execute it with substituted arguments

        if let Some(command_template) = self.simple_functions.get(function_name) {
            let command = self.substitute_args(command_template, args);
            return self.execute_command(&command);
        }

        // Check for full function definitions
        if let Some(body) = self.functions.get(function_name).cloned() {
            for stmt in body {
                self.execute_statement(stmt)?;
            }
            return Ok(());
        }

        Err(format!("Function '{}' not found", function_name).into())
    }

    fn substitute_args(&self, template: &str, args: &[String]) -> String {
        let mut result = template.to_string();

        // Replace $1, $2, $3, etc. with actual arguments
        for (i, arg) in args.iter().enumerate() {
            let placeholder = format!("${}", i + 1);
            result = result.replace(&placeholder, arg);
        }

        // Also support $@ for all arguments
        if result.contains("$@") {
            result = result.replace("$@", &args.join(" "));
        }

        // Replace user-defined variables (e.g., $myvar)
        for (var_name, var_value) in &self.variables {
            let placeholder = format!("${}", var_name);
            result = result.replace(&placeholder, var_value);
        }

        result
    }

    fn execute_statement(
        &mut self,
        statement: Statement,
    ) -> Result<(), Box<dyn std::error::Error>> {
        match statement {
            Statement::Assignment { name, value } => {
                let Expression::String(val) = value;
                self.variables.insert(name, val);
            }
            Statement::SimpleFunctionDef {
                name,
                command_template,
            } => {
                self.simple_functions.insert(name, command_template);
            }
            Statement::FunctionCall { name, args } => {
                // Call the function with the provided arguments
                self.call_function_with_args(&name, &args)?;
            }
            Statement::Command { command } => {
                // Substitute variables in the command before executing
                let substituted_command = self.substitute_args(&command, &[]);
                self.execute_command(&substituted_command)?;
            }
        }
        Ok(())
    }

    fn execute_command(&self, command: &str) -> Result<(), Box<dyn std::error::Error>> {
        // Check for RUN_SHELL environment variable, otherwise use platform defaults
        let shell_cmd = if let Ok(custom_shell) = std::env::var("RUN_SHELL") {
            custom_shell
        } else if cfg!(target_os = "windows") {
            // Default to bash on Windows
            // Try to find bash on PATH first, fallback to Git Bash default location
            if which::which("bash").is_ok() {
                "bash".to_string()
            } else {
                // Default Git Bash installation path
                r"C:\Program Files\Git\bin\bash.exe".to_string()
            }
        } else {
            // Default to sh on Unix-like systems
            "sh".to_string()
        };

        let status = Command::new(&shell_cmd)
            .arg("-c")
            .arg(command)
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()?;

        if !status.success() {
            eprintln!("Command failed with status: {}", status);
        }

        Ok(())
    }
}
