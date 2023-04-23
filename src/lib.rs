// use log::{debug, info, warn};
pub mod interpreter;
pub mod lex;
pub mod parse;

use interpreter::{Interpreter, RuntimeError};
use lex::Scanner;
use parse::{ParseError, Parser};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum LoxError {
    #[error("Parsing error: {}", 0)]
    Parse(ParseError),
    #[error("Runtime error: {}", 0)]
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

pub fn run(contents: &str) -> Result<(), LoxError> {
    let scanner = Scanner::new(contents);
    let mut parser = Parser::new(scanner);
    let statements = parser.parse()?;
    let interp = Interpreter::new();
    println!("{:?}", interp.interpret(&statements)?);
    Ok(())
}
