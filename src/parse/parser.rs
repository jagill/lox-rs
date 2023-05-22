use super::{BinaryOp, Expr, LogicalOp, ParseError, ParseResult, Stmt, UnaryOp};
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
        match self.peek_type() {
            None | Some(TokenType::Eof) => true,
            _ => false,
        }
    }

    pub fn parse(&mut self) -> ParseResult<Vec<Stmt>> {
        let mut statements: Vec<Stmt> = Vec::new();
        while !self.is_done() {
            statements.push(self.declaration()?);
        }
        Ok(statements)
    }

    pub fn declaration(&mut self) -> ParseResult<Stmt> {
        if self.match_next(TokenType::Var) {
            self.var_decl()
        } else if self.match_next(TokenType::Fun) {
            self.func_decl()
        } else {
            self.statement()
        }
    }

    fn var_decl(&mut self) -> ParseResult<Stmt> {
        let name = self.consume(TokenType::Identifier)?.lexeme.to_owned();
        let initializer = self
            .match_next(TokenType::Equal)
            .then(|| self.expression())
            .transpose()?;
        self.consume(TokenType::Semicolon)?;
        Ok(Stmt::Var { name, initializer })
    }

    fn func_decl(&mut self) -> ParseResult<Stmt> {
        let ident = self.consume(Identifier)?.lexeme.to_owned();

        // Params
        let mut params = Vec::new();
        self.consume(LeftParen)?;
        if !self.match_next(RightParen) {
            params.push(self.consume(Identifier)?.lexeme.to_owned());
            while let Some(token) = self.advance_only(Comma) {
                if params.len() >= 255 {
                    return Err(ParseError::TooManyArguments { line: token.line });
                }
                params.push(self.consume(Identifier)?.lexeme.to_owned());
            }
            self.consume(RightParen)?;
        }

        // Body
        self.consume(LeftBrace)?;
        let body = self.block_stmts()?;

        Ok(Stmt::Function {
            name: ident,
            params,
            body: body,
        })
    }

    pub fn statement(&mut self) -> ParseResult<Stmt> {
        use TokenType::*;

        let typ = self
            .advance_any_of(&[Print, LeftBrace, If, While, For, Return])
            .map(|t| t.typ);

        match typ {
            Some(Print) => {
                let value = self.expression()?;
                self.consume(TokenType::Semicolon)?;
                Ok(Stmt::Print(value))
            }
            Some(LeftBrace) => {
                let mut statements = Vec::new();
                while self.peek_type() != Some(TokenType::RightBrace) {
                    statements.push(self.declaration()?);
                }
                self.consume(TokenType::RightBrace)?;
                Ok(Stmt::Block(statements))
            }
            Some(If) => {
                self.consume(TokenType::LeftParen)?;
                let condition = self.expression()?;
                self.consume(TokenType::RightParen)?;

                let then_branch = self.statement()?;
                let else_branch = if self.match_next(TokenType::Else) {
                    Some(self.statement()?)
                } else {
                    None
                };

                Ok(Stmt::If {
                    condition,
                    then_branch: Box::new(then_branch),
                    else_branch: else_branch.map(Box::new),
                })
            }
            Some(While) => {
                self.consume(TokenType::LeftParen)?;
                let condition = self.expression()?;
                self.consume(TokenType::RightParen)?;
                let body = self.statement()?;

                Ok(Stmt::While {
                    condition,
                    body: Box::new(body),
                })
            }
            Some(For) => {
                self.consume(TokenType::LeftParen)?;

                let initializer: Option<Stmt> = if self.match_next(Semicolon) {
                    None
                } else if self.match_next(Var) {
                    Some(self.var_decl()?)
                } else {
                    Some(self.expr_stmt()?)
                };

                let condition = if self.match_next(Semicolon) {
                    None
                } else {
                    let expr = self.expression()?;
                    self.consume(Semicolon)?;
                    Some(expr)
                };

                let increment = if self.match_next(RightParen) {
                    None
                } else {
                    let expr = self.expression()?;
                    self.consume(RightParen)?;
                    Some(expr)
                };

                let mut body = self.statement()?;

                if let Some(incr) = increment {
                    body = Stmt::Block(vec![body, Stmt::Expression(incr)]);
                }

                body = Stmt::While {
                    condition: condition.unwrap_or(Expr::bool(true)),
                    body: Box::new(body),
                };

                if let Some(init) = initializer {
                    body = Stmt::Block(vec![init, body]);
                }

                Ok(body)
            }
            Some(Return) => {
                let expr = if self.peek_type() == Some(Semicolon) {
                    None
                } else {
                    Some(self.expression()?)
                };
                self.consume(Semicolon)?;
                Ok(Stmt::Return(expr))
            }
            // Expression statement
            _ => self.expr_stmt(),
        }
    }

    fn expr_stmt(&mut self) -> ParseResult<Stmt> {
        let expr = self.expression()?;
        self.consume(TokenType::Semicolon)?;
        Ok(Stmt::Expression(expr))
    }

    fn block_stmts(&mut self) -> ParseResult<Vec<Stmt>> {
        let mut statements = Vec::new();
        while self.peek_type() != Some(TokenType::RightBrace) {
            statements.push(self.declaration()?);
        }
        self.consume(TokenType::RightBrace)?;
        Ok(statements)
    }

    pub fn expression(&mut self) -> ParseResult<Expr> {
        self.assignment()
    }

    fn assignment(&mut self) -> ParseResult<Expr> {
        let expr = self.or()?;

        if let Some(token) = self.advance_only(Equal) {
            let line = token.line;
            let value = self.assignment()?;
            return if let Expr::Variable(name) = expr {
                Ok(Expr::assign(name, value))
            } else {
                Err(ParseError::InvalidAssignment { line })
            };
        }

        Ok(expr)
    }

    fn or(&mut self) -> ParseResult<Expr> {
        let mut expr = self.and()?;

        while self.match_next(Or) {
            expr = Expr::logical(expr, LogicalOp::Or, self.or()?);
        }

        Ok(expr)
    }

    fn and(&mut self) -> ParseResult<Expr> {
        let mut expr = self.equality()?;

        while self.match_next(And) {
            expr = Expr::logical(expr, LogicalOp::And, self.and()?);
        }

        Ok(expr)
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
            return Ok(Expr::binary(left, BinaryOp::Sub, self.term()?));
        }

        if self.match_next(Plus) {
            return Ok(Expr::binary(left, BinaryOp::Add, self.term()?));
        }

        Ok(left)
    }

    fn factor(&mut self) -> ParseResult<Expr> {
        let left = self.unary()?;

        if self.match_next(Slash) {
            return Ok(Expr::binary(left, BinaryOp::Div, self.factor()?));
        }

        if self.match_next(Star) {
            return Ok(Expr::binary(left, BinaryOp::Mult, self.factor()?));
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

        self.call()
    }

    fn call(&mut self) -> ParseResult<Expr> {
        let mut expr = self.primary()?;

        while self.match_next(LeftParen) {
            expr = self.finish_call(expr)?;
        }

        Ok(expr)
    }

    fn finish_call(&mut self, callee: Expr) -> ParseResult<Expr> {
        let mut args = Vec::new();
        if !self.match_next(RightParen) {
            args.push(self.expression()?);
            while let Some(comma) = self.advance_only(Comma) {
                if args.len() >= 255 {
                    return Err(ParseError::TooManyArguments { line: comma.line });
                }
                args.push(self.expression()?);
            }
            self.consume(RightParen)?;
        }

        Ok(Expr::Call {
            callee: Box::new(callee),
            args,
        })
    }

    fn primary(&mut self) -> ParseResult<Expr> {
        let primary_types = [Nil, False, True, Number, String_, Identifier, LeftParen];

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
            Identifier => Ok(Expr::var(token.lexeme)),
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

    fn advance_any_of(&mut self, typs: &[TokenType]) -> Option<Token> {
        if typs.contains(&self.peek()?.typ) {
            self.advance()
        } else {
            None
        }
    }

    /// Advance only if the next token is of the given type
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

    fn peek_type(&mut self) -> Option<TokenType> {
        self.tokens.peek().map(|t| t.typ)
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
        let actual = Parser::new(scanner).declaration();
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
    fn test_parse_var() {
        assert_parse_expr("abc", Ok(Expr::var("abc")))
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
            "(1 2",
            Err(ParseError::wrong_token(
                &Token::new(1, TokenType::Number, "2"),
                "token of type RightParen",
            )),
        );
    }

    #[test]
    fn test_parse_binary_num_ident() {
        assert_parse_expr(
            "1 <= a",
            Ok(Expr::binary(
                Expr::number(1.0),
                BinaryOp::LessEqual,
                Expr::var("a"),
            )),
        )
    }

    #[test]
    fn test_multiple_comparisons() {
        assert_parse_stmt(
            r#"1 < 2 < 3;"#,
            Err(ParseError::UnexpectedToken {
                actual: Less,
                line: 1,
                lexeme: "<".to_owned(),
                expected: String::new(),
            }),
        );

        assert_parse_expr(
            r#"1 < 2 == 3 > 4"#,
            Ok(Expr::binary(
                Expr::binary(Expr::number(1.0), BinaryOp::Less, Expr::number(2.0)),
                BinaryOp::Equal,
                Expr::binary(Expr::number(3.0), BinaryOp::Greater, Expr::number(4.0)),
            )),
        );

        assert_parse_stmt(
            r#"1 == 2 == 3;"#,
            Err(ParseError::UnexpectedToken {
                actual: EqualEqual,
                line: 1,
                lexeme: "=".to_owned(),
                expected: String::new(),
            }),
        );
    }

    #[test]
    fn test_multiple_terms() {
        assert_parse_expr(
            r#"1 + 2 + 3"#,
            Ok(Expr::binary(
                Expr::number(1.0),
                BinaryOp::Add,
                Expr::binary(Expr::number(2.0), BinaryOp::Add, Expr::number(3.0)),
            )),
        );

        assert_parse_expr(
            r#"1 * 2 * 3"#,
            Ok(Expr::binary(
                Expr::number(1.0),
                BinaryOp::Mult,
                Expr::binary(Expr::number(2.0), BinaryOp::Mult, Expr::number(3.0)),
            )),
        );

        assert_parse_expr(
            r#"1 + 2 * 3 + 4"#,
            Ok(Expr::binary(
                Expr::number(1.0),
                BinaryOp::Add,
                Expr::binary(
                    Expr::binary(Expr::number(2.0), BinaryOp::Mult, Expr::number(3.0)),
                    BinaryOp::Add,
                    Expr::number(4.0),
                ),
            )),
        );

        assert_parse_expr(
            r#"1 + 2 + 3 + 4"#,
            Ok(Expr::binary(
                Expr::number(1.0),
                BinaryOp::Add,
                Expr::binary(
                    Expr::number(2.0),
                    BinaryOp::Add,
                    Expr::binary(Expr::number(3.0), BinaryOp::Add, Expr::number(4.0)),
                ),
            )),
        );
    }

    #[test]
    fn test_string_concat_multiple_terms() {
        assert_parse_expr(
            r#""Hi, " + first + "!""#,
            Ok(Expr::binary(
                Expr::string("Hi, "),
                BinaryOp::Add,
                Expr::binary(Expr::var("first"), BinaryOp::Add, Expr::string("!")),
            )),
        );
    }

    #[test]
    fn test_parse_binary_logical() {
        assert_parse_expr(
            "true or false",
            Ok(Expr::logical(
                Expr::bool(true),
                LogicalOp::Or,
                Expr::bool(false),
            )),
        );

        assert_parse_expr(
            "true and false or x",
            Ok(Expr::logical(
                Expr::logical(Expr::bool(true), LogicalOp::And, Expr::bool(false)),
                LogicalOp::Or,
                Expr::var("x"),
            )),
        );

        assert_parse_expr(
            "a and b or c or d",
            Ok(Expr::logical(
                Expr::logical(Expr::var("a"), LogicalOp::And, Expr::var("b")),
                LogicalOp::Or,
                Expr::logical(Expr::var("c"), LogicalOp::Or, Expr::var("d")),
            )),
        )
    }

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

    #[test]
    fn test_parse_var_decl() {
        assert_parse_stmt(
            "var a = 1;",
            Ok(Stmt::Var {
                name: "a".to_owned(),
                initializer: Some(Expr::number(1.0)),
            }),
        );
        assert_parse_stmt(
            "var = 1;",
            Err(ParseError::UnexpectedToken {
                actual: TokenType::Equal,
                line: 1,
                lexeme: "=".to_owned(),
                expected: "".to_owned(),
            }),
        );
    }

    #[test]
    fn test_parse_var_use() {
        assert_parse_stmt("print a;", Ok(Stmt::Print(Expr::var("a"))));
        assert_parse_stmt(
            "print a",
            Err(ParseError::UnexpectedToken {
                actual: TokenType::Eof,
                line: 1,
                lexeme: "".to_owned(),
                expected: "".to_owned(),
            }),
        );
    }

    #[test]
    fn test_parse_assignment() {
        assert_parse_expr("a = 4", Ok(Expr::assign("a", Expr::number(4.))));
        assert_parse_expr("a + b = c", Err(ParseError::InvalidAssignment { line: 1 }));
    }

    #[test]
    fn test_parse_block() {
        assert_parse_stmt(
            "{ 1; 2;}",
            Ok(Stmt::Block(vec![
                Stmt::Expression(Expr::number(1.)),
                Stmt::Expression(Expr::number(2.)),
            ])),
        );
    }

    #[test]
    fn test_parse_nested_block() {
        assert_parse_stmt(
            "{ 1; { 2; } }",
            Ok(Stmt::Block(vec![
                Stmt::Expression(Expr::number(1.)),
                Stmt::Block(vec![Stmt::Expression(Expr::number(2.))]),
            ])),
        );
    }

    #[test]
    fn test_if_stmt() {
        assert_parse_stmt(
            "if (true) 1;",
            Ok(Stmt::If {
                condition: Expr::bool(true),
                then_branch: Box::new(Stmt::Expression(Expr::number(1.))),
                else_branch: None,
            }),
        );
        assert_parse_stmt(
            "if true 1;",
            Err(ParseError::UnexpectedToken {
                actual: TokenType::True,
                line: 1,
                lexeme: "true".to_owned(),
                expected: "".to_owned(),
            }),
        );
    }

    #[test]
    fn test_if_else_stmt() {
        assert_parse_stmt(
            "if (true) 1; else 2;",
            Ok(Stmt::If {
                condition: Expr::bool(true),
                then_branch: Box::new(Stmt::Expression(Expr::number(1.))),
                else_branch: Some(Box::new(Stmt::Expression(Expr::number(2.)))),
            }),
        );
        assert_parse_stmt(
            "if (true) 1; else 2",
            Err(ParseError::UnexpectedToken {
                actual: TokenType::Eof,
                line: 1,
                lexeme: "".to_owned(),
                expected: "".to_owned(),
            }),
        );
        assert_parse_stmt(
            "if (first) if (second) 1; else 2;",
            Ok(Stmt::If {
                condition: Expr::var("first"),
                then_branch: Box::new(Stmt::If {
                    condition: Expr::var("second"),
                    then_branch: Box::new(Stmt::Expression(Expr::number(1.))),
                    else_branch: Some(Box::new(Stmt::Expression(Expr::number(2.)))),
                }),
                else_branch: None,
            }),
        )
    }

    #[test]
    fn test_for_stmt() {
        assert_parse_stmt(
            r#"
            for (var i = 0; i < 20; i = i + 1) {
                print fib(i);
            }
            "#,
            Ok(Stmt::Block(vec![
                Stmt::Var {
                    name: "i".to_owned(),
                    initializer: Some(Expr::number(0.)),
                },
                Stmt::While {
                    condition: Expr::binary(Expr::var("i"), BinaryOp::Less, Expr::number(20.)),
                    body: Box::new(Stmt::Block(vec![
                        Stmt::Block(vec![Stmt::Print(Expr::Call {
                            callee: Box::new(Expr::var("fib")),
                            args: vec![Expr::var("i")],
                        })]),
                        Stmt::Expression(Expr::assign(
                            "i",
                            Expr::binary(Expr::var("i"), BinaryOp::Add, Expr::number(1.0)),
                        )),
                    ])),
                },
            ])),
        );
    }

    #[test]
    fn test_func_decl() {
        assert_parse_stmt(
            r#"fun sayHi(first, last) {
                print "Hi, " + first + "!";
            }
            "#,
            Ok(Stmt::Function {
                name: "sayHi".to_owned(),
                params: vec!["first".to_owned(), "last".to_owned()],
                body: vec![Stmt::Print(Expr::binary(
                    Expr::string("Hi, "),
                    BinaryOp::Add,
                    Expr::binary(Expr::var("first"), BinaryOp::Add, Expr::string("!")),
                ))],
            }),
        );
        assert_parse_expr(
            "callback()(foo)",
            Ok(Expr::Call {
                callee: Box::new(Expr::Call {
                    callee: Box::new(Expr::var("callback")),
                    args: Vec::new(),
                }),
                args: vec![Expr::var("foo")],
            }),
        );
    }

    #[test]
    fn test_if_return() {
        assert_parse_stmt(
            r#"
            if (n <= 1) return n;
        "#,
            Ok(Stmt::If {
                condition: Expr::binary(Expr::var("n"), BinaryOp::LessEqual, Expr::number(1.0)),
                then_branch: Box::new(Stmt::Return(Some(Expr::var("n")))),
                else_branch: None,
            }),
        );
    }

    #[test]
    fn test_func_decl_return() {
        assert_parse_stmt(
            r#"
        fun fib(n) {
            if (n <= 1) return n;
            return fib(n - 2) + fib(n - 1);
        }
        "#,
            Ok(Stmt::Function {
                name: "fib".to_owned(),
                params: vec!["n".to_owned()],
                body: vec![
                    Stmt::If {
                        condition: Expr::binary(
                            Expr::var("n"),
                            BinaryOp::LessEqual,
                            Expr::number(1.0),
                        ),
                        then_branch: Box::new(Stmt::Return(Some(Expr::var("n")))),
                        else_branch: None,
                    },
                    Stmt::Return(Some(Expr::binary(
                        Expr::Call {
                            callee: Box::new(Expr::var("fib")),
                            args: vec![Expr::binary(
                                Expr::var("n"),
                                BinaryOp::Sub,
                                Expr::number(2.0),
                            )],
                        },
                        BinaryOp::Add,
                        Expr::Call {
                            callee: Box::new(Expr::var("fib")),
                            args: vec![Expr::binary(
                                Expr::var("n"),
                                BinaryOp::Sub,
                                Expr::number(1.0),
                            )],
                        },
                    ))),
                ],
            }),
        );
    }
}
