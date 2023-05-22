use crate::interpreter::Interpreter;
use crate::parse::{Expr, ParseError, ParseResult, Stmt};
use std::collections::HashMap;

pub struct Resolver<'a> {
    interpreter: &'a mut Interpreter,
    scopes: Vec<HashMap<String, bool>>,
}

impl<'a> Resolver<'a> {
    pub fn new(interpreter: &'a mut Interpreter) -> Self {
        Self {
            interpreter,
            scopes: Vec::new(),
        }
    }

    fn begin_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn end_scope(&mut self) {
        self.scopes.pop();
    }

    fn declare(&mut self, name: &str) {
        self.scopes
            .last_mut()
            .map(|scope| scope.insert(name.to_owned(), false));
    }

    fn define(&mut self, name: &str) {
        self.scopes
            .last_mut()
            .map(|scope| scope.insert(name.to_owned(), true));
    }

    pub fn resolve_stmt(&mut self, stmt: &Stmt) -> ParseResult<()> {
        use Stmt::*;

        match stmt {
            Expression(expr) | Print(expr) | Return(Some(expr)) => self.resolve_expr(expr),
            If {
                condition,
                then_branch,
                else_branch,
            } => {
                self.resolve_expr(condition)?;
                self.resolve_stmt(then_branch)?;
                else_branch
                    .as_ref()
                    .map(|st| self.resolve_stmt(st))
                    .transpose()?;
                Ok(())
            }
            Var { name, initializer } => {
                self.declare(name);
                initializer
                    .as_ref()
                    .map(|init| self.resolve_expr(init))
                    .transpose()?;
                self.define(name);
                Ok(())
            }
            Function { name, params, body } => {
                self.declare(name);
                self.define(name);
                self.begin_scope();
                for param in params {
                    self.declare(param);
                    self.define(param);
                }
                for body_stmt in body {
                    self.resolve_stmt(body_stmt)?;
                }
                self.end_scope();
                Ok(())
            }
            Block(block_stmts) => {
                self.begin_scope();
                for block_stmt in block_stmts {
                    self.resolve_stmt(block_stmt)?;
                }
                self.end_scope();
                Ok(())
            }
            While { condition, body } => {
                self.resolve_expr(condition)?;
                self.resolve_stmt(body)
            }
            Return(None) => Ok(()),
        }
    }

    pub fn resolve_expr(&mut self, expr: &Expr) -> ParseResult<()> {
        use Expr::*;

        match expr {
            Unary { op: _, right } => self.resolve_expr(right),
            Binary { left, op: _, right } | Logical { left, op: _, right } => {
                self.resolve_expr(left)?;
                self.resolve_expr(right)
            }
            Grouping(e) => self.resolve_expr(e),
            Literal(_) => Ok(()),
            Variable(name) => self.resolve_local(name),
            Assign { name, expr: exp } => {
                self.resolve_expr(exp)?;
                self.resolve_local(name)
            }
            Call { callee, args } => {
                self.resolve_expr(callee)?;
                for arg in args {
                    self.resolve_expr(arg)?;
                }
                Ok(())
            }
        }
    }

    fn resolve_local(&mut self, name: &str) -> ParseResult<()> {
        if Some(&false) == self.scopes.last().and_then(|scope| scope.get(name)) {
            return Err(ParseError::SelfInitializedVar {
                name: name.to_owned(),
            });
        }
        for (idx, scope) in self.scopes.iter().enumerate().rev() {
            if scope.contains_key(name) {
                self.interpreter.resolve(name, idx);
                break;
            }
        }
        Ok(())
    }
}
