// Abstract Syntax Tree definitions

#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub statements: Vec<Statement>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    Assignment {
        name: String,
        value: Expression,
    },
    FunctionDef {
        name: String,
        body: Vec<Statement>,
    },
    FunctionCall {
        name: String,
    },
    Command {
        command: String,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    String(String),
    Number(i64),
    Identifier(String),
}

