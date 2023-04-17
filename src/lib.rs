// use log::{debug, info, warn};
mod errors;
pub mod lex;
pub mod parse;

use errors::LoxError;
use lex::Scanner;
use parse::Parser;

pub fn run(contents: &str) {
    let scanner = Scanner::new(contents);
    let mut parser = Parser::new(scanner);
    println!("{:?}", parser.parse());
}
