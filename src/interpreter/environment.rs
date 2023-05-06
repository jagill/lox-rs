use super::{RuntimeError, RuntimeResult, Value};
use std::collections::HashMap;

pub struct Environment {
    // A None enclosing means this is the top-level non-global scope
    enclosing: Option<Box<Environment>>,
    // Uninitialized variables (eg from `var x;`) are stored as Nil
    values: HashMap<String, Value>,
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

    pub fn define(&mut self, name: &str, value: Value) {
        self.values.insert(name.to_owned(), value);
    }

    pub fn get(&self, name: &str) -> RuntimeResult<&Value> {
        match (self.values.get(name), &self.enclosing) {
            (Some(val), _) => Ok(val),
            (None, Some(env)) => env.get(name),
            (None, None) => Err(RuntimeError::unbound_var(name)),
        }
    }

    pub fn assign(&mut self, name: &str, value: Value) -> RuntimeResult<()> {
        match (self.values.contains_key(name), &mut self.enclosing) {
            (true, _) => {
                self.values.insert(name.to_owned(), value);
                Ok(())
            }
            (false, Some(env)) => env.assign(&name, value),
            (false, None) => Err(RuntimeError::unbound_var(name)),
        }
    }

    /// Drop this environment and return the enclosing environment.
    pub fn pop(self) -> Option<Self> {
        self.enclosing.map(|e| *e)
    }
}
