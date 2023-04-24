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

    pub fn define(&mut self, name: impl ToString, value: Option<Value>) {
        let name = name.to_string();
        self.values.insert(name, value);
    }

    pub fn get(&self, name: &str) -> RuntimeResult<&Option<Value>> {
        let val = self.values.get(name);
        let res = val.ok_or(RuntimeError::unbound_var(name));
        res
    }
}
