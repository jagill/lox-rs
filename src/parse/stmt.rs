use super::Expr;

#[derive(Debug, PartialEq, Clone)]
pub enum Stmt {
    Expression(Expr),
    If {
        condition: Expr,
        then_branch: Box<Stmt>,
        else_branch: Option<Box<Stmt>>,
    },
    Print(Expr),
    Var {
        name: String,
        initializer: Option<Expr>,
    },
    Function {
        name: String,
        params: Vec<String>,
        body: Vec<Stmt>,
    },
    Block(Vec<Stmt>),
    While {
        condition: Expr,
        body: Box<Stmt>,
    },
    Return {
        expr: Option<Expr>,
    },
}
