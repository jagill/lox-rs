use std::fmt::{Display, Error as FmtError, Formatter};

use crate::parse::{Literal, LoxFunction};

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Nil,
    Bool(bool),
    Number(f64),
    String(String),
    Function(LoxFunction),
}

impl Value {
    pub fn of(lit: &Literal) -> Self {
        match lit {
            Literal::Nil => Value::Nil,
            Literal::Bool(b) => Value::Bool(*b),
            Literal::Number(num) => Value::Number(*num),
            Literal::String(s) => Value::String(s.to_owned()),
        }
    }

    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Nil => false,
            Value::Bool(b) => *b,
            _ => true,
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), FmtError> {
        match self {
            Self::Nil => write!(f, "nil"),
            Self::Bool(b) => write!(f, "{b}"),
            Self::Number(num) => write!(f, "{num}"),
            Self::String(s) => write!(f, "\"{s}\""),
            Self::Function(func) => {
                write!(f, "func {}(", func.name())?;
                for (idx, param) in func.params().iter().enumerate() {
                    if idx != 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{param}")?;
                }
                write!(f, ")")
            }
        }
    }
}
