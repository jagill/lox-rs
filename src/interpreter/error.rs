use thiserror::Error;

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
