use super::{RuntimeError, RuntimeResult, Value};
use std::collections::HashMap;

pub struct Environment {
    enclosing: Option<Box<Environment>>,
    values: HashMap<String, Option<Value>>,
}

impl Environment {
    pub fn new() -> Self {
        Environment {
            enclosing: None,
            values: HashMap::new(),
        }
    }

    pub fn with(enclosing: Environment) -> Self {
        Environment {
            enclosing: Some(Box::new(enclosing)),
            values: HashMap::new(),
        }
    }

    pub fn define(&mut self, name: &str, value: Option<Value>) {
        self.values.insert(name.to_owned(), value);
    }

    pub fn get(&self, name: &str) -> RuntimeResult<&Option<Value>> {
        match (self.values.get(name), &self.enclosing) {
            (Some(val), _) => Ok(val),
            (None, Some(env)) => env.get(name),
            (None, None) => Err(RuntimeError::unbound_var(name)),
        }
    }

    pub fn assign(&mut self, name: &str, value: Option<Value>) -> RuntimeResult<()> {
        if self.values.contains_key(name) {
            self.values.insert(name.to_owned(), value);
            Ok(())
        } else if let Some(env) = &mut self.enclosing {
            env.assign(&name, value)
        } else {
            Err(RuntimeError::unbound_var(name))
        }
    }

    /// Drop this environment and return the enclosing environment.
    pub fn pop(self) -> Option<Self> {
        self.enclosing.map(|e| *e)
    }
}
