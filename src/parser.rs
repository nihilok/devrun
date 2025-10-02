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
            let value = parse_expression(inner.next()?)?;
            Some(Statement::Assignment { name, value })
        }
        Rule::function_def => {
            let mut inner = pair.into_inner();
            let name = inner.next()?.as_str().to_string();
            let mut body = Vec::new();

            for stmt_pair in inner {
                match stmt_pair.as_rule() {
                    Rule::item => {
                        // Item wraps the actual content
                        if let Some(content) = stmt_pair.into_inner().next() {
                            match content.as_rule() {
                                Rule::comment => {
                                    // Skip comments
                                }
                                _ => {
                                    if let Some(stmt) = parse_statement(content) {
                                        body.push(stmt);
                                    }
                                }
                            }
                        }
                    }
                    _ => {
                        if let Some(stmt) = parse_statement(stmt_pair) {
                            body.push(stmt);
                        }
                    }
                }
            }

            Some(Statement::FunctionDef { name, body })
        }
        Rule::function_call => {
            let mut inner = pair.into_inner();
            let name = inner.next()?.as_str().to_string();
            Some(Statement::FunctionCall { name })
        }
        Rule::command => {
            let inner = pair.into_inner().next()?;
            let command = inner.as_str().trim().to_string();
            Some(Statement::Command { command })
        }
        _ => None,
    }
}

fn parse_expression(pair: pest::iterators::Pair<Rule>) -> Option<Expression> {
    match pair.as_rule() {
        Rule::expression => {
            let inner = pair.into_inner().next()?;
            parse_expression(inner)
        }
        Rule::string => {
            let inner = pair.into_inner().next()?;
            Some(Expression::String(inner.as_str().to_string()))
        }
        Rule::number => {
            let num = pair.as_str().parse::<i64>().ok()?;
            Some(Expression::Number(num))
        }
        Rule::identifier => {
            Some(Expression::Identifier(pair.as_str().to_string()))
        }
        _ => None,
    }
}
