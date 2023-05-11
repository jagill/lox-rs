use super::Stmt;

#[derive(Debug, Clone, PartialEq)]
pub struct LoxFunction {
    name: String,
    params: Vec<String>,
    body: Vec<Stmt>,
}

impl LoxFunction {
    pub fn new(name: String, params: Vec<String>, body: Vec<Stmt>) -> Self {
        Self { name, params, body }
    }
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn arity(&self) -> usize {
        self.params.len()
    }

    pub fn params(&self) -> &[String] {
        &self.params
    }

    pub fn body(&self) -> &[Stmt] {
        &self.body
    }
}
