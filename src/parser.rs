// Parser implementation using pest

use pest::Parser;
use pest_derive::Parser;
use crate::ast::{Program, Statement, Expression};

#[derive(Parser)]
#[grammar = "grammar.pest"]
pub struct ScriptParser;

pub fn parse_script(input: &str) -> Result<Program, Box<dyn std::error::Error>> {
    let pairs = ScriptParser::parse(Rule::program, input)?;
    let mut statements = Vec::new();

    for pair in pairs {
        match pair.as_rule() {
            Rule::program => {
                for inner_pair in pair.into_inner() {
                    match inner_pair.as_rule() {
                        Rule::item => {
                            // Item wraps the actual content
                            if let Some(content) = inner_pair.into_inner().next() {
                                match content.as_rule() {
                                    Rule::comment => {
                                        // Skip comments
                                    }
                                    _ => {
                                        if let Some(stmt) = parse_statement(content) {
                                            statements.push(stmt);
                                        }
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
            Rule::EOI => {}
            _ => {}
        }
    }

    Ok(Program { statements })
}

fn parse_statement(pair: pest::iterators::Pair<Rule>) -> Option<Statement> {
    match pair.as_rule() {
        Rule::assignment => {
            let mut inner = pair.into_inner();
            let name = inner.next()?.as_str().to_string();
            let value_str = inner.next()?.as_str().to_string();
            Some(Statement::Assignment {
                name,
                value: Expression::String(value_str)
            })
        }
        Rule::function_def => {
            let mut inner = pair.into_inner();
            let name = inner.next()?.as_str().to_string();

            // The next element is the command
            if let Some(cmd_pair) = inner.next() {
                let command_template = parse_command(cmd_pair);
                Some(Statement::SimpleFunctionDef { name, command_template })
            } else {
                None
            }
        }
        Rule::function_call => {
            let mut inner = pair.into_inner();
            let name = inner.next()?.as_str().to_string();
            // TODO: Handle arguments in function calls
            Some(Statement::FunctionCall { name })
        }
        Rule::command => {
            let command = parse_command(pair);
            Some(Statement::Command { command })
        }
        _ => None,
    }
}

fn parse_command(pair: pest::iterators::Pair<Rule>) -> String {
    let mut result = String::new();

    for part in pair.into_inner() {
        match part.as_rule() {
            Rule::quoted_string => {
                result.push('"');
                result.push_str(part.as_str().trim_matches('"'));
                result.push('"');
            }
            Rule::variable => {
                result.push_str(part.as_str());
            }
            Rule::operator => {
                result.push(' ');
                result.push_str(part.as_str());
                result.push(' ');
            }
            Rule::word => {
                result.push_str(part.as_str());
            }
            _ => {
                result.push_str(part.as_str());
            }
        }
        result.push(' ');
    }

    result.trim().to_string()
}
