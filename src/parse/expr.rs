#[derive(Clone, Debug)]
pub enum Expr {
    Unary {
        op: UnaryOp,
        right: Box<Expr>,
    },
    Binary {
        left: Box<Expr>,
        op: BinaryOp,
        right: Box<Expr>,
    },
    Grouping(Box<Expr>),
    Literal(Literal),
}

#[derive(Clone, Debug)]
pub enum Literal {
    Nil,
    Bool(bool),
    Number(f64),
    String(String),
}

impl Expr {
    pub fn nil() -> Self {
        Expr::Literal(Literal::Nil)
    }
    pub fn bool(b: bool) -> Self {
        Expr::Literal(Literal::Bool(b))
    }
    pub fn number(f: f64) -> Self {
        Expr::Literal(Literal::Number(f))
    }
    pub fn string(s: &str) -> Self {
        Expr::Literal(Literal::String(s.to_owned()))
    }

    pub fn unary(op: UnaryOp, right: Expr) -> Self {
        Expr::Unary {
            op,
            right: Box::new(right),
        }
    }

    pub fn binary(left: Expr, op: BinaryOp, right: Expr) -> Self {
        Expr::Binary {
            left: Box::new(left),
            op,
            right: Box::new(right),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum BinaryOp {
    Mult,
    Div,
    Add,
    Sub,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
    NotEqual,
    Equal,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum UnaryOp {
    Not,
    Minus,
}