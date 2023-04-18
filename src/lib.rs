// use log::{debug, info, warn};
mod errors;
pub mod interpreter;
pub mod lex;
pub mod parse;

use errors::LoxError;
use interpreter::Interpreter;
use lex::Scanner;
use parse::Parser;

type LoxResult<T> = Result<T, LoxError>;

pub fn run(contents: &str) -> LoxResult<()> {
    let scanner = Scanner::new(contents);
    let mut parser = Parser::new(scanner);
    let ast = parser.parse()?;
    let interp = Interpreter::new();
    println!("{:?}", interp.interpret(&ast)?);
    Ok(())
}
