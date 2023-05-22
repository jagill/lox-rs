// use log::{debug, info, warn};
pub mod interpreter;
pub mod lex;
pub mod parse;
pub mod analysis;

use interpreter::{Interpreter, RuntimeError};
use lex::Scanner;
use parse::Stmt;
use parse::{ParseError, Parser};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum LoxError {
    #[error("Parsing error: {0}")]
    Parse(ParseError),
    #[error("Runtime error: {0}")]
    Runtime(RuntimeError),
}

impl From<ParseError> for LoxError {
    fn from(err: ParseError) -> Self {
        Self::Parse(err)
    }
}

impl From<RuntimeError> for LoxError {
    fn from(err: RuntimeError) -> Self {
        Self::Runtime(err)
    }
}

pub struct Lox {
    interpreter: Interpreter,
}

impl Lox {
    pub fn new() -> Self {
        Self {
            interpreter: Interpreter::new(),
        }
    }

    pub fn run(&mut self, contents: &str) -> Result<(), LoxError> {
        let statements = self.parse(contents)?;
        self.interpret(&statements)?;
        Ok(())
    }

    fn parse(&self, contents: &str) -> Result<Vec<Stmt>, ParseError> {
        let scanner = Scanner::new(contents);
        let mut parser = Parser::new(scanner);
        parser.parse()
    }

    fn interpret(&mut self, statements: &[Stmt]) -> Result<(), RuntimeError> {
        self.interpreter.interpret(statements)
    }
}
