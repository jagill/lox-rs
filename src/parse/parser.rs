use super::{BinaryOp, Expr, UnaryOp};
use crate::lex::{Scanner, Token, TokenType, TokenType::*};
use std::iter::Peekable;

pub struct Parser<'a> {
    tokens: Peekable<Scanner<'a>>, // tokens: Vec<Token<'a>>,
                                   // current: usize,
}

impl<'a> Parser<'a> {
    pub fn new(scanner: Scanner<'a>) -> Self {
        // let tokens: Vec<Token<'a>> = scanner.collect();
        // Self { tokens, current: 0 }
        let tokens = scanner.peekable();
        Self { tokens }
    }

    pub fn parse(&mut self) -> Option<Expr> {
        self.expression()
    }

    fn expression(&mut self) -> Option<Expr> {
        self.equality()
    }

    fn equality(&mut self) -> Option<Expr> {
        let left = self.comparison()?;

        if self.match_next(BangEqual) {
            return Some(Expr::binary(left, BinaryOp::NotEqual, self.comparison()?));
        }

        if self.match_next(EqualEqual) {
            return Some(Expr::binary(left, BinaryOp::Equal, self.comparison()?));
        }

        Some(left)
    }

    fn comparison(&mut self) -> Option<Expr> {
        let left = self.term()?;

        if self.match_next(Greater) {
            return Some(Expr::binary(left, BinaryOp::Greater, self.term()?));
        }

        if self.match_next(GreaterEqual) {
            return Some(Expr::binary(left, BinaryOp::GreaterEqual, self.term()?));
        }

        if self.match_next(Less) {
            return Some(Expr::binary(left, BinaryOp::Less, self.term()?));
        }

        if self.match_next(LessEqual) {
            return Some(Expr::binary(left, BinaryOp::LessEqual, self.term()?));
        }

        Some(left)
    }

    fn term(&mut self) -> Option<Expr> {
        let left = self.factor()?;

        if self.match_next(Minus) {
            return Some(Expr::binary(left, BinaryOp::Sub, self.factor()?));
        }

        if self.match_next(Plus) {
            return Some(Expr::binary(left, BinaryOp::Add, self.factor()?));
        }

        Some(left)
    }

    fn factor(&mut self) -> Option<Expr> {
        let left = self.unary()?;

        if self.match_next(Slash) {
            return Some(Expr::binary(left, BinaryOp::Div, self.unary()?));
        }

        if self.match_next(Star) {
            return Some(Expr::binary(left, BinaryOp::Mult, self.unary()?));
        }

        Some(left)
    }

    fn unary(&mut self) -> Option<Expr> {
        if self.match_next(Bang) {
            return Some(Expr::unary(UnaryOp::Not, self.unary()?));
        }
        if self.match_next(Minus) {
            return Some(Expr::unary(UnaryOp::Minus, self.unary()?));
        }
        self.primary()
    }

    fn primary(&mut self) -> Option<Expr> {
        let primary_types = [Nil, False, True, Number, String_, LeftParen];

        let token = self.advance_if(|token| primary_types.contains(&token.typ))?;
        match token.typ {
            Nil => Some(Expr::nil()),
            False => Some(Expr::bool(false)),
            True => Some(Expr::bool(true)),
            Number => {
                let num: f64 = token.lexeme().parse().unwrap();
                Some(Expr::number(num))
            }
            String_ => {
                let val = token
                    .lexeme()
                    .strip_prefix(r#"""#)
                    .expect(r#"String lexeme should start with " "#)
                    .strip_suffix(r#"""#)
                    .expect(r#"String lexeme should end with " "#);
                Some(Expr::string(val))
            }
            LeftParen => {
                let expr = self.expression();
                // TODO: Handle this with error
                self.advance_if(|token| token.typ == RightParen)
                    .expect("Unmatched Parenthesis!");
                expr
            }
            _ => None,
        }
    }

    fn advance(&mut self) -> Option<Token> {
        self.tokens.next()
    }

    fn advance_if(&mut self, pred: impl FnOnce(&Token<'a>) -> bool) -> Option<Token> {
        if pred(self.peek()?) {
            self.advance()
        } else {
            None
        }
    }

    fn advance_only(&mut self, typ: TokenType) -> Option<Token> {
        if self.peek()?.typ == typ {
            self.advance()
        } else {
            None
        }
    }

    fn match_next(&mut self, typ: TokenType) -> bool {
        self.advance_only(typ).is_some()
    }

    fn peek(&mut self) -> Option<&Token<'a>> {
        self.tokens.peek()
    }
}
