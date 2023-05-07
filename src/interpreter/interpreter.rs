use std::collections::HashMap;

use super::Value;
use super::{Environment, RuntimeError, RuntimeResult};
use crate::parse::{BinaryOp, Expr, LogicalOp, Stmt, UnaryOp};

pub struct Interpreter {}

impl Interpreter {
    pub fn new() -> Self {
        Self {}
    }

    pub fn interpret(&self, stmts: &[Stmt]) -> RuntimeResult<()> {
        // TODO: Replace this with one that is populated.
        let mut env = Environment::with_globals(HashMap::new());
        for stmt in stmts {
            self.statement(stmt, &mut env)?;
        }
        Ok(())
    }

    pub fn statement(&self, stmt: &Stmt, env: &mut Environment) -> RuntimeResult<()> {
        match stmt {
            Stmt::Var { name, initializer } => {
                let value = initializer
                    .as_ref()
                    .map(|expr| self.expression(expr, env))
                    .transpose()?;
                env.define(name, value.unwrap_or(Value::Nil))
            }
            Stmt::Expression(expr) => self.expression(expr, env).map(|_| ()),
            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => {
                if self.expression(condition, env)?.is_truthy() {
                    self.statement(&then_branch, env)
                } else if let Some(else_br) = else_branch {
                    self.statement(else_br, env)
                } else {
                    Ok(())
                }
            }
            Stmt::Print(expr) => {
                let value = self.expression(expr, env)?;
                println!("{}", value);
                Ok(())
            }
            Stmt::Block(statements) => {
                env.push();
                let mut res: RuntimeResult<()> = Ok(());
                for stmt in statements {
                    res = self.statement(stmt, env);
                    if res.is_err() {
                        break;
                    }
                }
                env.pop();
                res
            }
            Stmt::While { condition, body } => {
                while self.expression(condition, env)?.is_truthy() {
                    self.statement(body, env)?;
                }
                Ok(())
            }
        }
    }

    pub fn expression(&self, expr: &Expr, env: &mut Environment) -> RuntimeResult<Value> {
        match expr {
            Expr::Literal(lit) => Ok(Value::of(lit)),
            Expr::Grouping(expr) => self.expression(expr, env),
            Expr::Unary { op, right } => {
                let value = self.expression(right, env)?;
                self.unary(*op, &value)
            }
            Expr::Binary { left, op, right } => {
                let left_val = self.expression(left, env)?;
                let right_val = self.expression(right, env)?;
                self.binary(&left_val, *op, &right_val)
            }
            Expr::Variable(name) => env.get(name).map(|v| v.clone()),
            Expr::Assign { name, expr } => {
                let val = self.expression(expr, env)?;
                env.assign(name, val.clone())?;
                Ok(val)
            }
            Expr::Logical { left, op, right } => {
                let left_val = self.expression(left, env)?;
                match (left_val.is_truthy(), op) {
                    (true, LogicalOp::Or) | (false, LogicalOp::And) => Ok(left_val),
                    (false, LogicalOp::Or) | (true, LogicalOp::And) => self.expression(right, env),
                }
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
        let interp = Interpreter::new();
        let mut env = Environment::with_globals(HashMap::new());
        let scanner = Scanner::new(source);
        let ast = Parser::new(scanner).expression().unwrap();
        let actual = interp.expression(&ast, &mut env);
        match (&actual, &expected) {
            (Err(actual_err), Err(expected_err)) => assert_eq!(
                std::mem::discriminant(actual_err),
                std::mem::discriminant(expected_err)
            ),
            _ => assert_eq!(actual, expected),
        }
    }

    fn assert_statement(source: &str, success: bool) {
        let interp = Interpreter::new();
        let mut env = Environment::with_globals(HashMap::new());
        let scanner = Scanner::new(source);
        let ast = Parser::new(scanner).declaration().unwrap();
        let actual = interp.statement(&ast, &mut env);
        match (success, actual.is_ok()) {
            (true, true) => (),
            (false, false) => (),
            (true, false) => assert_eq!(Ok(()), actual),
            (false, true) => panic!("Interpret should fail, but succeeded."),
        }
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

    #[test]
    fn test_interpret_blocks() {
        assert_statement(
            r#"
        var a = "global a";
        var b = "global b";
        var c = "global c";
        {
            var a = "outer a";
            var b = "outer b";
            {
                var a = "inner a";
                print a;
                print b;
                print c;
            }
            print a;
            print b;
            print c;
        }
        print a;
        print b;
        print c;
        "#,
            true,
        );
    }

    #[test]
    fn test_if_stmt() {
        assert_statement("if (true) 1;", true);
        assert_statement("if (true) 1; else 2;", true);
        assert_statement("if (true) if (true) 1; else 2;", true)
    }

    #[test]
    fn test_logical_expr() {
        assert_expression(r#" "hi" or 2 "#, Ok(Value::String("hi".to_owned())));
        assert_expression(r#" "hi" and 2 "#, Ok(Value::Number(2.)));
        assert_expression(r#" nil or "yes" "#, Ok(Value::String("yes".to_owned())));
        assert_expression(r#" nil and "yes" "#, Ok(Value::Nil));
    }

    #[test]
    fn test_loop_stmt() {
        assert_statement("var going = true; while (going) going = false;", true);
        assert_statement(
            r#"
        var a = 0;
        var temp;
        for (var b = 1; a < 10000; b = temp + b) {
            print a;
            temp = a;
            a = b;
        }
        "#,
            true,
        );
    }
}
