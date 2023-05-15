use thiserror::Error;

use super::Value;

#[derive(Debug, Error, PartialEq)]
pub enum RuntimeError {
    #[error("Type error: {expected}.")]
    TypeError { expected: String },
    #[error("Unbound variable: {name}.")]
    UnboundVar { name: String },
    #[error("Trying to assign an existing global variable: {name}.")]
    AssigningGlobal { name: String },
    #[error("Trying to define a global variable: {name}.")]
    DefiningGlobal { name: String },
    #[error("Trying to call {name} (arity {arity}) with {num_args} arguments.")]
    ArityMismatch {
        name: String,
        arity: usize,
        num_args: usize,
    },
    #[error("Trying to call a non-callable value: {value}")]
    CallingNonCallable { value: Value },
    #[error("Return statement not in function call")]
    Return { value: Value },
}

impl RuntimeError {
    pub fn type_error(msg: impl Into<String>) -> Self {
        RuntimeError::TypeError {
            expected: msg.into(),
        }
    }
    pub fn unbound_var(name: impl Into<String>) -> Self {
        RuntimeError::UnboundVar { name: name.into() }
    }
    pub fn assigning_global(name: impl Into<String>) -> Self {
        RuntimeError::AssigningGlobal { name: name.into() }
    }
}
