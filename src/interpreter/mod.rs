mod error;
mod interpreter;
mod value;

pub use error::RuntimeError;
pub use interpreter::Interpreter;
pub use value::Value;

type RuntimeResult<T> = Result<T, RuntimeError>;
