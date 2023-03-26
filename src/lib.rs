// use log::{debug, info, warn};
pub mod lex;
use lex::Scanner;

pub fn run(contents: &str) {
    let scanner = Scanner::new(contents);
    for token in scanner {
        println!("Got {:?}", token);
    }
}