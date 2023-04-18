use super::{BinaryOp, Expr, UnaryOp};
use crate::lex::{Scanner, Token, TokenType, TokenType::*};
use crate::{LoxError, LoxResult};
use std::iter::Peekable;

pub struct Parser<'a> {
    tokens: Peekable<Scanner<'a>>,
}

impl<'a> Parser<'a> {
    pub fn new(scanner: Scanner<'a>) -> Self {
        let tokens = scanner.peekable();
        Self { tokens }
    }

    pub fn parse(&mut self) -> LoxResult<Expr> {
        self.expression()
    }

    pub fn expression(&mut self) -> LoxResult<Expr> {
        self.equality()
    }

    fn equality(&mut self) -> LoxResult<Expr> {
        let left = self.comparison()?;

        if self.match_next(BangEqual) {
            return Ok(Expr::binary(left, BinaryOp::NotEqual, self.comparison()?));
        }

        if self.match_next(EqualEqual) {
            return Ok(Expr::binary(left, BinaryOp::Equal, self.comparison()?));
        }

        Ok(left)
    }

    fn comparison(&mut self) -> LoxResult<Expr> {
        let left = self.term()?;

        if self.match_next(Greater) {
            return Ok(Expr::binary(left, BinaryOp::Greater, self.term()?));
        }

        if self.match_next(GreaterEqual) {
            return Ok(Expr::binary(left, BinaryOp::GreaterEqual, self.term()?));
        }

        if self.match_next(Less) {
            return Ok(Expr::binary(left, BinaryOp::Less, self.term()?));
        }

        if self.match_next(LessEqual) {
            return Ok(Expr::binary(left, BinaryOp::LessEqual, self.term()?));
        }

        Ok(left)
    }

    fn term(&mut self) -> LoxResult<Expr> {
        let left = self.factor()?;

        if self.match_next(Minus) {
            return Ok(Expr::binary(left, BinaryOp::Sub, self.factor()?));
        }

        if self.match_next(Plus) {
            return Ok(Expr::binary(left, BinaryOp::Add, self.factor()?));
        }

        Ok(left)
    }

    fn factor(&mut self) -> LoxResult<Expr> {
        let left = self.unary()?;

        if self.match_next(Slash) {
            return Ok(Expr::binary(left, BinaryOp::Div, self.unary()?));
        }

        if self.match_next(Star) {
            return Ok(Expr::binary(left, BinaryOp::Mult, self.unary()?));
        }

        Ok(left)
    }

    fn unary(&mut self) -> LoxResult<Expr> {
        if self.match_next(Bang) {
            return Ok(Expr::unary(UnaryOp::Not, self.unary()?));
        }
        if self.match_next(Minus) {
            return Ok(Expr::unary(UnaryOp::Minus, self.unary()?));
        }
        self.primary()
    }

    fn primary(&mut self) -> LoxResult<Expr> {
        let primary_types = [Nil, False, True, Number, String_, LeftParen];

        let token = self.advance_expect("primary expression", |token| {
            primary_types.contains(&token.typ)
        })?;
        match token.typ {
            Nil => Ok(Expr::nil()),
            False => Ok(Expr::bool(false)),
            True => Ok(Expr::bool(true)),
            Number => {
                let num: f64 = token.lexeme.parse().unwrap();
                Ok(Expr::number(num))
            }
            String_ => Ok(Expr::string(token.lexeme)),
            LeftParen => {
                let expr = self.expression()?;
                self.consume(RightParen)?;
                Ok(Expr::group(expr))
            }
            _ => Err(LoxError::wrong_token(&token, "expression")),
        }
    }

    fn advance(&mut self) -> Option<Token> {
        self.tokens.next()
    }

    fn advance_expect(
        &mut self,
        message: &str,
        pred: impl FnOnce(&Token<'a>) -> bool,
    ) -> LoxResult<Token> {
        let token = self.peek().ok_or(LoxError::end(message))?;
        if pred(token) {
            Ok(self.advance().unwrap())
        } else {
            Err(LoxError::wrong_token(token, message))
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

    fn consume(&mut self, typ: TokenType) -> LoxResult<Token> {
        self.advance_expect(&format!("token of type {typ:?}"), |t| t.typ == typ)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lex::Scanner;

    fn assert_parse_expr(source: &str, expected: Result<Expr, LoxError>) {
        let scanner = Scanner::new(source);
        let actual = Parser::new(scanner).expression();
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_parse_number() {
        assert_parse_expr("1", Ok(Expr::number(1.0)))
    }

    #[test]
    fn test_parse_bool() {
        assert_parse_expr("true", Ok(Expr::bool(true)))
    }

    #[test]
    fn test_parse_unary_num() {
        assert_parse_expr("-1", Ok(Expr::unary(UnaryOp::Minus, Expr::number(1.0))))
    }

    #[test]
    fn test_parse_unary_bool() {
        assert_parse_expr("!false", Ok(Expr::unary(UnaryOp::Not, Expr::bool(false))))
    }

    #[test]
    fn test_parse_binary_num() {
        assert_parse_expr(
            "1 + 2",
            Ok(Expr::binary(
                Expr::number(1.0),
                BinaryOp::Add,
                Expr::number(2.0),
            )),
        )
    }

    #[test]
    fn test_parse_binary_bool() {
        assert_parse_expr(
            "true != false",
            Ok(Expr::binary(
                Expr::bool(true),
                BinaryOp::NotEqual,
                Expr::bool(false),
            )),
        );
    }

    #[test]
    fn test_parse_precedence() {
        assert_parse_expr(
            "1 + 2 * 3",
            Ok(Expr::binary(
                Expr::number(1.),
                BinaryOp::Add,
                Expr::binary(Expr::number(2.), BinaryOp::Mult, Expr::number(3.)),
            )),
        );

        assert_parse_expr(
            "1 * 2 + 3",
            Ok(Expr::binary(
                Expr::binary(Expr::number(1.), BinaryOp::Mult, Expr::number(2.)),
                BinaryOp::Add,
                Expr::number(3.),
            )),
        );

        assert_parse_expr(
            "1 + 2 > 3",
            Ok(Expr::binary(
                Expr::binary(Expr::number(1.), BinaryOp::Add, Expr::number(2.)),
                BinaryOp::Greater,
                Expr::number(3.),
            )),
        );

        assert_parse_expr(
            "1 <= 2 - 3",
            Ok(Expr::binary(
                Expr::number(1.),
                BinaryOp::LessEqual,
                Expr::binary(Expr::number(2.), BinaryOp::Sub, Expr::number(3.)),
            )),
        );
    }

    #[test]
    fn test_parse_grouping() {
        assert_parse_expr(
            "(1 + 2) * 3",
            Ok(Expr::binary(
                Expr::group(Expr::binary(
                    Expr::number(1.),
                    BinaryOp::Add,
                    Expr::number(2.),
                )),
                BinaryOp::Mult,
                Expr::number(3.),
            )),
        );
    }

    #[test]
    fn test_parse_string() {
        assert_parse_expr(r#""abc""#, Ok(Expr::string("abc")));
        assert_parse_expr("\"a\nb\"", Ok(Expr::string("a\nb")));
    }

    #[test]
    fn test_parse_error_eof() {
        assert_parse_expr(
            "(",
            Err(LoxError::wrong_token(
                &Token::new(1, TokenType::Eof, ""),
                "primary expression",
            )),
        );

        assert_parse_expr(
            "abc",
            Err(LoxError::wrong_token(
                &Token::new(1, TokenType::Identifier, "abc"),
                "primary expression",
            )),
        );

        assert_parse_expr(
            "(1 2",
            Err(LoxError::wrong_token(
                &Token::new(1, TokenType::Number, "2"),
                "token of type RightParen",
            )),
        );
    }

    // TODO: Not yet implemented
    // #[test]
    // fn test_parse_binary_num_ident() {
    //     assert_parse_expr(
    //         "1 <= a",
    //         Ok(Expr::binary(
    //             Expr::number(1.0),
    //             BinaryOp::LessEqual,
    //             Expr::identifier("a"),
    //         )),
    //     )
    // }

    // TODO: Not yet implemented
    // #[test]
    // fn test_parse_binary_bool() {
    //     assert_parse_expr(
    //         "true or false",
    //         Ok(Expr::binary(
    //             Expr::bool(true),
    //             BinaryOp::Or,
    //             Expr::bool(false),
    //         )),
    //     )
    // }
}
