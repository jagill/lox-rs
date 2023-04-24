use super::Value;
use super::{Environment, RuntimeError, RuntimeResult};
use crate::parse::{BinaryOp, Expr, Stmt, UnaryOp};

pub struct Interpreter {
    env: Environment,
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            env: Environment::new(),
        }
    }

    pub fn interpret(&mut self, stmts: &[Stmt]) -> RuntimeResult<()> {
        for stmt in stmts {
            self.statement(stmt)?;
        }
        Ok(())
    }

    pub fn statement(&mut self, stmt: &Stmt) -> RuntimeResult<()> {
        match stmt {
            Stmt::Var { name, initializer } => {
                let value = initializer
                    .as_ref()
                    .map(|expr| self.expression(expr))
                    .transpose()?;
                self.env.define(name, value);
            }
            Stmt::Expression(expr) => {
                self.expression(expr)?;
            }
            Stmt::Print(expr) => {
                let value = self.expression(expr)?;
                println!("{}", value);
            }
        }
        Ok(())
    }

    pub fn expression(&mut self, expr: &Expr) -> RuntimeResult<Value> {
        match expr {
            Expr::Literal(lit) => Ok(Value::of(lit)),
            Expr::Grouping(expr) => self.expression(expr),
            Expr::Unary { op, right } => {
                let value = self.expression(right)?;
                self.unary(*op, &value)
            }
            Expr::Binary { left, op, right } => {
                let left_val = self.expression(left)?;
                let right_val = self.expression(right)?;
                self.binary(&left_val, *op, &right_val)
            }
            Expr::Variable(name) => {
                let val_opt = self.env.get(name)?.as_ref();
                let val = val_opt.map_or(Value::Nil, |v| v.clone());
                Ok(val)
            }
            Expr::Assign { name, expr } => {
                let val = self.expression(expr)?;
                self.env.assign(name, Some(val.clone()))?;
                Ok(val)
            }
        }
    }

    fn unary(&self, op: UnaryOp, value: &Value) -> RuntimeResult<Value> {
        match (op, value) {
            (UnaryOp::Minus, Value::Number(num)) => Ok(Value::Number(-*num)),
            (UnaryOp::Not, val) => Ok(Value::Bool(!val.is_truthy())),
            _ => Err(RuntimeError::type_error(format!(
                "Can't combine {op:?} and {value:?}"
            ))),
        }
    }

    fn binary(&self, left_val: &Value, op: BinaryOp, right_val: &Value) -> RuntimeResult<Value> {
        match (left_val, op, right_val) {
            (left, BinaryOp::Equal, right) => Ok(Value::Bool(left == right)),
            (left, BinaryOp::NotEqual, right) => Ok(Value::Bool(left != right)),
            (Value::Number(left), BinaryOp::Mult, Value::Number(right)) => {
                Ok(Value::Number(left * right))
            }
            (Value::Number(left), BinaryOp::Div, Value::Number(right)) => {
                Ok(Value::Number(left / right))
            }
            (Value::Number(left), BinaryOp::Add, Value::Number(right)) => {
                Ok(Value::Number(left + right))
            }
            (Value::Number(left), BinaryOp::Sub, Value::Number(right)) => {
                Ok(Value::Number(left - right))
            }
            (Value::String(left), BinaryOp::Add, Value::String(right)) => {
                Ok(Value::String(format!("{left}{right}")))
            }
            (Value::Number(left), BinaryOp::Greater, Value::Number(right)) => {
                Ok(Value::Bool(left > right))
            }
            (Value::Number(left), BinaryOp::GreaterEqual, Value::Number(right)) => {
                Ok(Value::Bool(left >= right))
            }
            (Value::Number(left), BinaryOp::Less, Value::Number(right)) => {
                Ok(Value::Bool(left < right))
            }
            (Value::Number(left), BinaryOp::LessEqual, Value::Number(right)) => {
                Ok(Value::Bool(left <= right))
            }
            _ => Err(RuntimeError::type_error(format!(
                "Can't combine {left_val:?} and {right_val:?} with {op:?}"
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{lex::Scanner, parse::Parser};

    use super::*;

    fn assert_expression(source: &str, expected: RuntimeResult<Value>) {
        let mut interp = Interpreter::new();
        let scanner = Scanner::new(source);
        let ast = Parser::new(scanner).expression().unwrap();
        let actual = interp.expression(&ast);
        match (&actual, &expected) {
            (Err(actual_err), Err(expected_err)) => assert_eq!(
                std::mem::discriminant(actual_err),
                std::mem::discriminant(expected_err)
            ),
            _ => assert_eq!(actual, expected),
        }
    }

    fn assert_statement(source: &str, success: bool) {
        let mut interp = Interpreter::new();
        let scanner = Scanner::new(source);
        let ast = Parser::new(scanner).declaration().unwrap();
        let actual = interp.statement(&ast);
        assert_eq!(actual.is_ok(), success);
    }

    #[test]
    fn test_interpret_literals() {
        assert_expression("1", Ok(Value::Number(1.)));
        assert_expression("false", Ok(Value::Bool(false)));
        assert_expression(r#""abc""#, Ok(Value::String("abc".to_owned())));
    }

    #[test]
    fn test_interpret_group() {
        assert_expression("(1)", Ok(Value::Number(1.)));
    }

    #[test]
    fn test_interpret_unary() {
        assert_expression("-1", Ok(Value::Number(-1.)));
        assert_expression("!true", Ok(Value::Bool(false)));
        assert_expression("!nil", Ok(Value::Bool(true)));
        assert_expression("-false", Err(RuntimeError::type_error("")));
    }

    #[test]
    fn test_interpret_binary() {
        assert_expression("1-1", Ok(Value::Number(0.)));
        assert_expression("true != false", Ok(Value::Bool(true)));
        assert_expression("true == 1", Ok(Value::Bool(false)));
        assert_expression("2 >= 1", Ok(Value::Bool(true)));
        assert_expression("2 * 1.01", Ok(Value::Number(2.02)));
        assert_expression(r#""a" + "b""#, Ok(Value::String("ab".to_owned())));
        assert_expression("1 + false", Err(RuntimeError::type_error("")));
    }

    #[test]
    fn test_interpret_complex() {
        assert_expression("2 > (2 * 1.01)", Ok(Value::Bool(false)));
    }

    #[test]
    fn test_interpret_statement_expr() {
        assert_statement("1;", true);
        assert_statement("print 1;", true);
    }

    #[test]
    fn test_interpret_statements() {
        assert_statement(
            r#"
        print "one"; print true; print 2 + 1;
        "#,
            true,
        );
        assert_statement(
            r#"
        var a = 1;
        var b = 2;
        print a + b;
        "#,
            true,
        );
    }
}
