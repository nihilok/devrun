// Interpreter to execute the AST

use crate::ast::{Program, Statement, Expression};
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

    pub fn call_function_without_parens(&mut self, function_name: &str, args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
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
        if with_colons != function_name {
            if let Some(command_template) = self.simple_functions.get(&with_colons) {
                let command = self.substitute_args(command_template, args);
                return self.execute_command(&command);
            }
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

    pub fn call_function(&mut self, function_name: &str, args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
        // Try to match the function name directly
        if let Some(command_template) = self.simple_functions.get(function_name) {
            let command = self.substitute_args(command_template, args);
            return self.execute_command(&command);
        }

        // Try to match with space-separated nested commands (e.g., "docker shell" -> "docker:shell")
        let nested_name = function_name.replace(" ", ":");
        if let Some(command_template) = self.simple_functions.get(&nested_name) {
            let command = self.substitute_args(command_template, args);
            return self.execute_command(&command);
        }

        // Try matching the first part with remaining as subcommands
        let parts: Vec<&str> = function_name.split_whitespace().collect();
        if parts.len() > 1 {
            let nested_with_args = format!("{}:{}", parts[0], parts[1..].join(":"));
            if let Some(command_template) = self.simple_functions.get(&nested_with_args) {
                let command = self.substitute_args(command_template, args);
                return self.execute_command(&command);
            }
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

        result
    }

    fn execute_statement(&mut self, statement: Statement) -> Result<(), Box<dyn std::error::Error>> {
        match statement {
            Statement::Assignment { name, value } => {
                let val = self.evaluate_expression(value)?;
                self.variables.insert(name, val);
            }
            Statement::FunctionDef { name, body } => {
                self.functions.insert(name, body);
            }
            Statement::SimpleFunctionDef { name, command_template } => {
                self.simple_functions.insert(name, command_template);
            }
            Statement::FunctionCall { name } => {
                // First check simple function definitions
                if let Some(command_template) = self.simple_functions.get(&name) {
                    let command = command_template.clone();
                    self.execute_command(&command)?;
                } else if let Some(body) = self.functions.get(&name).cloned() {
                    for stmt in body {
                        self.execute_statement(stmt)?;
                    }
                } else {
                    eprintln!("Error: Function '{}' not defined", name);
                }
            }
            Statement::Command { command } => {
                self.execute_command(&command)?;
            }
        }
        Ok(())
    }

    fn evaluate_expression(&self, expr: Expression) -> Result<String, Box<dyn std::error::Error>> {
        match expr {
            Expression::String(s) => Ok(s),
            Expression::Number(n) => Ok(n.to_string()),
            Expression::Identifier(name) => {
                Ok(self.variables.get(&name).cloned().unwrap_or_default())
            }
        }
    }

    fn execute_command(&self, command: &str) -> Result<(), Box<dyn std::error::Error>> {
        let output = if cfg!(target_os = "windows") {
            Command::new("cmd")
                .args(["/C", command])
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .output()?
        } else {
            Command::new("sh")
                .arg("-c")
                .arg(command)
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .output()?
        };

        if !output.status.success() {
            eprintln!("Command failed with status: {}", output.status);
        }

        Ok(())
    }
}
