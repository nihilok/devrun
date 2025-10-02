// Interpreter to execute the AST

use crate::ast::{Program, Statement, Expression};
use std::collections::HashMap;
use std::process::{Command, Stdio};

pub struct Interpreter {
    variables: HashMap<String, String>,
    functions: HashMap<String, Vec<Statement>>,
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
            functions: HashMap::new(),
        }
    }

    pub fn execute(&mut self, program: Program) -> Result<(), Box<dyn std::error::Error>> {
        for statement in program.statements {
            self.execute_statement(statement)?;
        }
        Ok(())
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
            Statement::FunctionCall { name } => {
                if let Some(body) = self.functions.get(&name).cloned() {
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
