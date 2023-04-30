#[derive(Clone, Debug, PartialEq)]
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
    Variable(String),
    Assign {
        name: String,
        expr: Box<Expr>,
    },
    Logical {
        left: Box<Expr>,
        op: LogicalOp,
        right: Box<Expr>,
    },
}

#[derive(Clone, Debug, PartialEq)]
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
    pub fn var(s: &str) -> Self {
        Expr::Variable(s.to_owned())
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

    pub fn group(expr: Expr) -> Self {
        Expr::Grouping(Box::new(expr))
    }

    pub fn assign(name: impl Into<String>, expr: Expr) -> Self {
        Expr::Assign {
            name: name.into(),
            expr: Box::new(expr),
        }
    }

    pub fn logical(left: Expr, op: LogicalOp, right: Expr) -> Self {
        Expr::Logical {
            left: Box::new(left),
            op,
            right: Box::new(right),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
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

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum UnaryOp {
    Not,
    Minus,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum LogicalOp {
    And,
    Or,
}
