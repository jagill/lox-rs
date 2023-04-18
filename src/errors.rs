use crate::lex::{Token, TokenType};
use thiserror::Error;

#[derive(Debug, Error, PartialEq)]
pub enum LoxError {
    #[error("Expected {expected}, but ran out of tokens.")]
    UnexpectedEnd { expected: String },
    #[error("Expected {expected} on line {line}, but found {actual:?} '{lexeme}'.")]
    UnexpectedToken {
        actual: TokenType,
        line: usize,
        lexeme: String,
        expected: String,
    },
    #[error("Type error: {expected}.")]
    TypeError { expected: String },
}

impl LoxError {
    pub fn end(msg: impl Into<String>) -> Self {
        LoxError::UnexpectedEnd {
            expected: msg.into(),
        }
    }

    pub fn wrong_token(token: &Token<'_>, msg: impl Into<String>) -> Self {
        LoxError::UnexpectedToken {
            actual: token.typ,
            line: token.line,
            lexeme: token.lexeme.to_owned(),
            expected: msg.into(),
        }
    }

    pub fn type_error(msg: impl Into<String>) -> Self {
        LoxError::TypeError { expected: msg.into() }
    }
}
