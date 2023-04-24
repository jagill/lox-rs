use super::{RuntimeError, RuntimeResult, Value};
use std::collections::HashMap;

pub struct Environment {
    values: HashMap<String, Option<Value>>,
}

impl Environment {
    pub fn new() -> Self {
        Environment {
            values: HashMap::new(),
        }
    }

    pub fn define(&mut self, name: &str, value: Option<Value>) {
        self.values.insert(name.to_owned(), value);
    }

    pub fn get(&self, name: &str) -> RuntimeResult<&Option<Value>> {
        let val = self.values.get(name);
        let res = val.ok_or(RuntimeError::unbound_var(name));
        res
    }

    pub fn assign(&mut self, name: &str, value: Option<Value>) -> RuntimeResult<()> {
        if self.values.contains_key(name) {
            self.values.insert(name.to_owned(), value);
            Ok(())
        } else {
            Err(RuntimeError::unbound_var(name))
        }
    }
}
