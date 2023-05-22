mod error;
mod expr;
mod function;
mod parser;
mod stmt;

pub use error::ParseError;
pub use expr::{BinaryOp, Expr, Literal, LogicalOp, UnaryOp};
pub use function::LoxFunction;
pub use parser::Parser;
pub use stmt::Stmt;

pub type ParseResult<T> = Result<T, ParseError>;
