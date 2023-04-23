use thiserror::Error;

#[derive(Debug, Error, PartialEq)]
pub enum RuntimeError {
    #[error("Type error: {expected}.")]
    TypeError { expected: String },
}

impl RuntimeError {
    pub fn type_error(msg: impl Into<String>) -> Self {
        RuntimeError::TypeError {
            expected: msg.into(),
        }
    }
}
