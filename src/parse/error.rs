use crate::lex::{Token, TokenType};
use thiserror::Error;

#[derive(Debug, Error, PartialEq)]
pub enum ParseError {
    #[error("Expected {expected}, but ran out of tokens.")]
    UnexpectedEnd { expected: String },
    #[error("Expected {expected} on line {line}, but found {actual:?} '{lexeme}'.")]
    UnexpectedToken {
        actual: TokenType,
        line: usize,
        lexeme: String,
        expected: String,
    },
    #[error("Invalid assignment target on line {line}.")]
    InvalidAssignment { line: usize },
    #[error("Arguments to a function are capped at 255 (line {line})")]
    TooManyArguments { line: usize },
}

impl ParseError {
    pub fn end(msg: impl Into<String>) -> Self {
        ParseError::UnexpectedEnd {
            expected: msg.into(),
        }
    }

    pub fn wrong_token(token: &Token<'_>, msg: impl Into<String>) -> Self {
        ParseError::UnexpectedToken {
            actual: token.typ,
            line: token.line,
            lexeme: token.lexeme.to_owned(),
            expected: msg.into(),
        }
    }
}
