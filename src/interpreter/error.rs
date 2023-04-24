use thiserror::Error;

#[derive(Debug, Error, PartialEq)]
pub enum RuntimeError {
    #[error("Type error: {expected}.")]
    TypeError { expected: String },
    #[error("Unbound variable: {name}.")]
    UnboundVar { name: String },
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
}
