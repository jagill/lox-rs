use super::{BinaryOp, Expr, ParseError, ParseResult, Stmt, UnaryOp};
use crate::lex::{Scanner, Token, TokenType, TokenType::*};
use std::iter::Peekable;

pub struct Parser<'a> {
    tokens: Peekable<Scanner<'a>>,
}

impl<'a> Parser<'a> {
    pub fn new(scanner: Scanner<'a>) -> Self {
        let tokens = scanner.peekable();
        Self { tokens }
    }

    fn is_done(&mut self) -> bool {
        match self.peek().map(|t| t.typ) {
            None | Some(TokenType::Eof) => true,
            _ => false,
        }
    }

    pub fn parse(&mut self) -> ParseResult<Vec<Stmt>> {
        let mut statements: Vec<Stmt> = Vec::new();
        while !self.is_done() {
            statements.push(self.statement()?);
        }
        Ok(statements)
    }

    pub fn statement(&mut self) -> ParseResult<Stmt> {
        if self.match_next(TokenType::Print) {
            let value = self.expression()?;
            self.consume(TokenType::Semicolon)?;
            Ok(Stmt::Print(value))
        } else {
            let expr = self.expression()?;
            self.consume(TokenType::Semicolon)?;
            Ok(Stmt::Expression(expr))
        }
    }

    pub fn expression(&mut self) -> ParseResult<Expr> {
        self.equality()
    }

    fn equality(&mut self) -> ParseResult<Expr> {
        let left = self.comparison()?;

        if self.match_next(BangEqual) {
            return Ok(Expr::binary(left, BinaryOp::NotEqual, self.comparison()?));
        }

        if self.match_next(EqualEqual) {
            return Ok(Expr::binary(left, BinaryOp::Equal, self.comparison()?));
        }

        Ok(left)
    }

    fn comparison(&mut self) -> ParseResult<Expr> {
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

    fn term(&mut self) -> ParseResult<Expr> {
        let left = self.factor()?;

        if self.match_next(Minus) {
            return Ok(Expr::binary(left, BinaryOp::Sub, self.factor()?));
        }

        if self.match_next(Plus) {
            return Ok(Expr::binary(left, BinaryOp::Add, self.factor()?));
        }

        Ok(left)
    }

    fn factor(&mut self) -> ParseResult<Expr> {
        let left = self.unary()?;

        if self.match_next(Slash) {
            return Ok(Expr::binary(left, BinaryOp::Div, self.unary()?));
        }

        if self.match_next(Star) {
            return Ok(Expr::binary(left, BinaryOp::Mult, self.unary()?));
        }

        Ok(left)
    }

    fn unary(&mut self) -> ParseResult<Expr> {
        if self.match_next(Bang) {
            return Ok(Expr::unary(UnaryOp::Not, self.unary()?));
        }
        if self.match_next(Minus) {
            return Ok(Expr::unary(UnaryOp::Minus, self.unary()?));
        }
        self.primary()
    }

    fn primary(&mut self) -> ParseResult<Expr> {
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
            _ => Err(ParseError::wrong_token(&token, "expression")),
        }
    }

    fn advance(&mut self) -> Option<Token> {
        self.tokens.next()
    }

    fn advance_expect(
        &mut self,
        message: &str,
        pred: impl FnOnce(&Token<'a>) -> bool,
    ) -> ParseResult<Token> {
        let token = self.peek().ok_or(ParseError::end(message))?;
        if pred(token) {
            Ok(self.advance().unwrap())
        } else {
            Err(ParseError::wrong_token(token, message))
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

    fn consume(&mut self, typ: TokenType) -> ParseResult<Token> {
        self.advance_expect(&format!("token of type {typ:?}"), |t| t.typ == typ)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lex::Scanner;

    fn assert_parse_expr(source: &str, expected: Result<Expr, ParseError>) {
        let scanner = Scanner::new(source);
        let actual = Parser::new(scanner).expression();
        match (&actual, &expected) {
            (Err(actual_err), Err(expected_err)) => assert_eq!(
                std::mem::discriminant(actual_err),
                std::mem::discriminant(expected_err)
            ),
            _ => assert_eq!(actual, expected),
        }
    }

    fn assert_parse_stmt(source: &str, expected: Result<Stmt, ParseError>) {
        let scanner = Scanner::new(source);
        let actual = Parser::new(scanner).statement();
        match (&actual, &expected) {
            (Err(actual_err), Err(expected_err)) => assert_eq!(
                std::mem::discriminant(actual_err),
                std::mem::discriminant(expected_err)
            ),
            _ => assert_eq!(actual, expected),
        }
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
            Err(ParseError::wrong_token(
                &Token::new(1, TokenType::Eof, ""),
                "primary expression",
            )),
        );

        assert_parse_expr(
            "abc",
            Err(ParseError::wrong_token(
                &Token::new(1, TokenType::Identifier, "abc"),
                "primary expression",
            )),
        );

        assert_parse_expr(
            "(1 2",
            Err(ParseError::wrong_token(
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

    #[test]
    fn test_parse_stmt_expr() {
        assert_parse_stmt("1;", Ok(Stmt::Expression(Expr::number(1.))));
        assert_parse_stmt(
            "1",
            Err(ParseError::UnexpectedToken {
                actual: TokenType::Eof,
                line: 1,
                lexeme: "".to_owned(),
                expected: "".to_owned(),
            }),
        );
    }

    #[test]
    fn test_parse_stmt_print() {
        assert_parse_stmt("print 1;", Ok(Stmt::Print(Expr::number(1.))));
        assert_parse_stmt(
            "print 1",
            Err(ParseError::UnexpectedToken {
                actual: TokenType::Eof,
                line: 1,
                lexeme: "".to_owned(),
                expected: "".to_owned(),
            }),
        );
        assert_parse_stmt(
            "print print",
            Err(ParseError::UnexpectedToken {
                actual: TokenType::Print,
                line: 1,
                lexeme: "print".to_owned(),
                expected: "".to_owned(),
            }),
        );
    }
}
