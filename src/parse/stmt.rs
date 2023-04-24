use super::Expr;

#[derive(Debug, PartialEq)]
pub enum Stmt {
    Expression(Expr),
    Print(Expr),
    Var {
        name: String,
        initializer: Option<Expr>,
    },
    Block(Vec<Stmt>),
}
