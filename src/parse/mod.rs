mod error;
mod expr;
mod parser;

pub use error::ParseError;
pub use expr::{BinaryOp, Expr, Literal, UnaryOp};
pub use parser::Parser;

type ParseResult<T> = Result<T, ParseError>;
