use super::{RuntimeError, RuntimeResult, Value};
use std::{collections::HashMap, rc::Rc};

pub struct GlobalEnvironment {
    values: HashMap<String, Value>,
}

pub struct LocalEnvironment {
    enclosing: Box<Environment>,
    values: HashMap<String, Value>,
}

pub enum Environment {
    Local(LocalEnvironment),
    Global(Rc<GlobalEnvironment>),
}

impl Environment {
    pub fn with_globals(values: HashMap<String, Value>) -> Self {
        let mut env = Environment::Global(Rc::new(GlobalEnvironment { values }));
        env.push();
        env
    }

    pub fn local(enclosing: Environment) -> Self {
        Environment::Local(LocalEnvironment {
            enclosing: Box::new(enclosing),
            values: HashMap::new(),
        })
    }

    // Hack to allow push/pop
    fn dummy() -> Self {
        Environment::Global(Rc::new(GlobalEnvironment {
            values: HashMap::new(),
        }))
    }

    pub fn globals(&self) -> Environment {
        match self {
            Environment::Global(env) => {
                let mut env = Environment::Global(env.clone());
                env.push();
                env
            }
            Environment::Local(env) => env.enclosing.globals(),
        }
    }

    pub fn define(&mut self, name: &str, value: Value) -> RuntimeResult<()> {
        match self {
            Environment::Global(_) => Err(RuntimeError::DefiningGlobal {
                name: name.to_owned(),
            }),
            Environment::Local(env) => {
                env.values.insert(name.to_owned(), value);
                Ok(())
            }
        }
    }

    /// If the symbol isn't found, return None
    pub fn get(&self, name: &str) -> RuntimeResult<&Value> {
        match self {
            Environment::Global(env) => env.values.get(name).ok_or(RuntimeError::unbound_var(name)),
            Environment::Local(env) => match env.values.get(name) {
                Some(val) => Ok(val),
                None => env.enclosing.get(name),
            },
        }
    }

    // Return if the assign was successful
    pub fn assign(&mut self, name: &str, value: Value) -> RuntimeResult<()> {
        match self {
            Environment::Global(env) => {
                if env.values.contains_key(name) {
                    Err(RuntimeError::assigning_global(name))
                } else {
                    Err(RuntimeError::unbound_var(name))
                }
            }
            Environment::Local(env) => {
                if env.values.contains_key(name) {
                    env.values.insert(name.to_owned(), value);
                    Ok(())
                } else {
                    env.enclosing.assign(name, value)
                }
            }
        }
    }

    /// Push a local environment on top of this environment.
    pub fn push(&mut self) {
        let env = std::mem::replace(self, Self::dummy());
        *self = Environment::local(env);
    }

    /// Drop this environment and return the enclosing environment.
    pub fn pop(&mut self) {
        let env = std::mem::replace(self, Self::dummy());
        match env {
            Environment::Global(_) => panic!("Attempted to pop a global environment"),
            Environment::Local(local) => *self = *local.enclosing,
        }
    }
}
