use super::Value;
use crate::parse::{Expr, BinaryOp, UnaryOp};
use crate::{LoxError, LoxResult};

pub struct Interpreter {}

impl Interpreter {
    pub fn new() -> Self {
        Interpreter {}
    }

    pub fn interpret(&self, expr: &Expr) -> LoxResult<Value> {
        match expr {
            Expr::Literal(lit) => Ok(Value::of(lit)),
            Expr::Grouping(expr) => self.interpret(expr),
            Expr::Unary { op, right } => {
                let value = self.interpret(right)?;
                self.unary(*op, &value)
            }
            Expr::Binary { left, op, right } => {
                let left_val = self.interpret(left)?;
                let right_val = self.interpret(right)?;
                self.binary(&left_val, *op, &right_val)
            }
        }
    }

    fn unary(&self, op: UnaryOp, value: &Value) -> LoxResult<Value> {
        match (op, value) {
            (UnaryOp::Minus, Value::Number(num)) => Ok(Value::Number(-*num)),
            (UnaryOp::Not, val) => Ok(Value::Bool(!val.is_truthy())),
            _ => Err(LoxError::type_error(format!(
                "Can't combine {op:?} and {value:?}"
            ))),
        }
    }

    fn binary(&self, left_val: &Value, op: BinaryOp, right_val: &Value) -> LoxResult<Value> {
        match (left_val, op, right_val) {
            (left, BinaryOp::Equal, right) => Ok(Value::Bool(left == right)),
            (left, BinaryOp::NotEqual, right) => Ok(Value::Bool(left != right)),
            (Value::Number(left), BinaryOp::Mult, Value::Number(right)) => Ok(Value::Number(
                left * right
            )),
            (Value::Number(left), BinaryOp::Div, Value::Number(right)) => Ok(Value::Number(
                left / right
            )),
            (Value::Number(left), BinaryOp::Add, Value::Number(right)) => Ok(Value::Number(
                left + right
            )),
            (Value::Number(left), BinaryOp::Sub, Value::Number(right)) => Ok(Value::Number(
                left - right
            )),
            (Value::String(left), BinaryOp::Add, Value::String(right)) => Ok(Value::String(
                format!("{left}{right}")
            )),
            (Value::Number(left), BinaryOp::Greater, Value::Number(right)) => Ok(Value::Bool(
                left > right
            )),
            (Value::Number(left), BinaryOp::GreaterEqual, Value::Number(right)) => Ok(Value::Bool(
                left >= right
            )),
            (Value::Number(left), BinaryOp::Less, Value::Number(right)) => Ok(Value::Bool(
                left < right
            )),
            (Value::Number(left), BinaryOp::LessEqual, Value::Number(right)) => Ok(Value::Bool(
                left <= right
            )),
            _ => Err(LoxError::type_error(format!(
                "Can't combine {left_val:?} and {right_val:?} with {op:?}"
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{lex::Scanner, parse::Parser};

    use super::*;

    fn assert_value(source: &str, expected: LoxResult<Value>) {
        let interp = Interpreter::new();
        let scanner = Scanner::new(source);
        let ast = Parser::new(scanner).expression().unwrap();
        let actual = interp.interpret(&ast);
        match (&actual, &expected) {
            (Err(actual_err), Err(expected_err)) => assert_eq!(
                std::mem::discriminant(actual_err),
                std::mem::discriminant(expected_err)
            ),
            _ => assert_eq!(actual, expected),
        }
    }

    #[test]
    fn test_interpret_literals() {
        assert_value("1", Ok(Value::Number(1.)));
        assert_value("false", Ok(Value::Bool(false)));
        assert_value(r#""abc""#, Ok(Value::String("abc".to_owned())));
    }

    #[test]
    fn test_interpret_group() {
        assert_value("(1)", Ok(Value::Number(1.)));
    }

    #[test]
    fn test_interpret_unary() {
        assert_value("-1", Ok(Value::Number(-1.)));
        assert_value("!true", Ok(Value::Bool(false)));
        assert_value("!nil", Ok(Value::Bool(true)));
        assert_value("-false", Err(LoxError::type_error("")));
    }

    #[test]
    fn test_interpret_binary() {
        assert_value("1-1", Ok(Value::Number(0.)));
        assert_value("true != false", Ok(Value::Bool(true)));
        assert_value("true == 1", Ok(Value::Bool(false)));
        assert_value("2 >= 1", Ok(Value::Bool(true)));
        assert_value("2 * 1.01", Ok(Value::Number(2.02)));
        assert_value(r#""a" + "b""#, Ok(Value::String("ab".to_owned())));
        assert_value("1 + false", Err(LoxError::type_error("")));
    }

    #[test]
    fn test_interpret_complex() {
        assert_value("2 > (2 * 1.01)", Ok(Value::Bool(false)));
    }
}
