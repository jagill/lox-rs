mod error;
mod expr;
mod parser;
mod stmt;

pub use error::ParseError;
pub use expr::{BinaryOp, Expr, Literal, LogicalOp, UnaryOp};
pub use parser::Parser;
pub use stmt::Stmt;

type ParseResult<T> = Result<T, ParseError>;
