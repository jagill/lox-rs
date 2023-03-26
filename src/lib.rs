// use log::{debug, info, warn};
pub mod lex;
pub mod parse;

use lex::Scanner;
use parse::Parser;

pub fn run(contents: &str) {
    let scanner = Scanner::new(contents);
    let mut parser = Parser::new(scanner);
    println!("{:?}", parser.parse());
}
