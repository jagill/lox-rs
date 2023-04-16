use super::{Token, TokenType};
use TokenType::*;

pub struct Scanner<'a> {
    source: &'a str,
    char_idxs: std::iter::Peekable<std::str::CharIndices<'a>>,
    start: usize,
    current: usize,
    line: usize,
    eof: bool,
}

impl<'a> Scanner<'a> {
    pub fn new(source: &'a str) -> Self {
        Scanner {
            source,
            char_idxs: source.char_indices().peekable(),
            start: 0,
            current: 0,
            line: 1,
            eof: false,
        }
    }

    fn scan_token(&mut self) -> Option<Token<'a>> {
        self.skip_whitespace();
        self.start = self.current;

        let ch = self.advance()?;
        let typ = match ch {
            '(' => LeftParen,
            ')' => RightParen,
            '{' => LeftBrace,
            '}' => RightBrace,
            ',' => Comma,
            '.' => Dot,
            '-' => Minus,
            '+' => Plus,
            ';' => Semicolon,
            '*' => Star,

            '!' => {
                if self.match_next('=') {
                    BangEqual
                } else {
                    Bang
                }
            }
            '=' => {
                if self.match_next('=') {
                    EqualEqual
                } else {
                    Equal
                }
            }
            '<' => {
                if self.match_next('=') {
                    LessEqual
                } else {
                    Less
                }
            }
            '>' => {
                if self.match_next('=') {
                    GreaterEqual
                } else {
                    Greater
                }
            }
            '/' => {
                if self.match_next('/') {
                    self.advance_while(|ch| ch != '\n');
                    Comment
                } else {
                    Slash
                }
            }

            // String literals, because they can embed newlines and have to be
            // stripped of their surrounding double-quotes, need special handling.
            '"' => {
                // Skip initial '"'
                self.start = self.current;
                let starting_line = self.line;
                self.advance_while(|ch| ch != '"');
                let mut token = Token::new(starting_line, String_, self.current_lexeme());
                // Skip final '"', but check to make sure everything's ok.
                match self.advance() {
                    // The expected case
                    Some('"') => (),
                    // This is a bug in our parser, because advance_while should ensure this never happens,
                    Some(_) => {
                        let current_line = self.line;
                        panic!(
                            r#"String literal parsing finished but next character is not a '"'.
                        Started on line {starting_line}, error at line {current_line}.
                        "#
                        );
                    }
                    // Eof before string literal closed.
                    None => {
                        token = Token::new(self.line, ErrorUnclosedString, self.current_lexeme());
                    }
                }
                return Some(token);
            }

            '0'..='9' => self.advance_number(),

            'A'..='Z' | 'a'..='z' | '_' => {
                self.advance_while(is_kw_char);
                TokenType::get(self.current_lexeme())
            }

            _ => ErrorUnknownToken,
        };

        let lexeme = self.current_lexeme();
        Some(Token::new(self.line, typ, lexeme))
    }

    fn skip_whitespace(&mut self) {
        let whitespace_chars = [' ', '\n', '\t', '\r'];
        self.advance_while(|ch| whitespace_chars.contains(&ch));
    }

    fn current_lexeme(&self) -> &'a str {
        &self.source[self.start..self.current]
    }

    // Advance past the rest of the number. It assumes the first digit has already been consumed.
    fn advance_number(&mut self) -> TokenType {
        let pred = |ch| '0' <= ch && ch <= '9';
        self.advance_while(pred);

        // If there's a period, keep matching digits for a float.
        // If there are no digits, that's a lex error.
        if self.match_next('.') {
            if self.advance_if(pred).is_some() {
                self.advance_while(pred);
            } else {
                return TokenType::ErrorMalformedNumber;
            }
        }
        TokenType::Number
    }

    /// Advance to the next char, if any
    fn advance(&mut self) -> Option<char> {
        let (_idx, ch) = self.char_idxs.next()?;
        if ch == '\n' {
            self.line += 1;
        }
        self.current = self.next_idx();
        Some(ch)
    }

    /// Advance if the next character matches the predicate.
    fn advance_if(&mut self, pred: impl Fn(char) -> bool) -> Option<char> {
        match self.peek()? {
            next_ch if pred(next_ch) => self.advance(),
            _ => None,
        }
    }

    /// Keep advancing while the predicate is true
    fn advance_while<F>(&mut self, pred: F)
    where
        F: Copy + Fn(char) -> bool,
    {
        while self.advance_if(pred).is_some() {}
    }

    fn next_idx(&mut self) -> usize {
        self.char_idxs.peek().map_or(self.source.len(), |(i, _)| *i)
    }

    /// If the next char is equal to `ch`, advance and return true.  Else, return false.
    fn match_next(&mut self, ch: char) -> bool {
        self.advance_if(|next_ch| ch == next_ch).is_some()
    }

    /// Return the next char without advancing.
    fn peek(&mut self) -> Option<char> {
        self.char_idxs.peek().map(|(_i, c)| *c)
    }
}

impl<'a> Iterator for Scanner<'a> {
    type Item = Token<'a>;

    fn next(&mut self) -> Option<Token<'a>> {
        if self.eof {
            None
        } else {
            self.scan_token().or_else(|| {
                self.eof = true;
                Some(Token {
                    line: self.line,
                    typ: Eof,
                    lexeme: "",
                })
            })
        }
    }
}

fn is_kw_char(ch: char) -> bool {
    ('A' <= ch && ch <= 'Z') || ('a' <= ch && ch <= 'z') || ('0' <= ch && ch <= '9') || ch == '_'
}

#[cfg(test)]
mod tests {
    use super::{Scanner, Token, TokenType};

    fn assert_scan(source: &str, expected: Vec<Token>) {
        let tokens: Vec<Token> = Scanner::new(source).collect();
        assert_eq!(tokens, expected);
    }

    #[test]
    fn test_empty_scan() {
        assert_scan("", vec![Token::new(1, TokenType::Eof, "")]);
    }

    #[test]
    fn test_empty_whitespace_scan() {
        assert_scan("  \t ", vec![Token::new(1, TokenType::Eof, "")]);
    }

    #[test]
    fn test_empty_whitespace_with_newline_scan() {
        assert_scan("  \n\r  ", vec![Token::new(2, TokenType::Eof, "")]);
    }

    #[test]
    fn test_function_scan() {
        assert_scan(
            r#"fun printSum(a, b) {
            print a + b;
        }
        "#,
            vec![
                Token::new(1, TokenType::Fun, "fun"),
                Token::new(1, TokenType::Identifier, "printSum"),
                Token::new(1, TokenType::LeftParen, "("),
                Token::new(1, TokenType::Identifier, "a"),
                Token::new(1, TokenType::Comma, ","),
                Token::new(1, TokenType::Identifier, "b"),
                Token::new(1, TokenType::RightParen, ")"),
                Token::new(1, TokenType::LeftBrace, "{"),
                Token::new(2, TokenType::Print, "print"),
                Token::new(2, TokenType::Identifier, "a"),
                Token::new(2, TokenType::Plus, "+"),
                Token::new(2, TokenType::Identifier, "b"),
                Token::new(2, TokenType::Semicolon, ";"),
                Token::new(3, TokenType::RightBrace, "}"),
                Token::new(4, TokenType::Eof, ""),
            ],
        );
    }

    #[test]
    fn test_string_literal_scan() {
        assert_scan(
            r#""this is a fun 'literal'""#,
            vec![
                Token::new(1, TokenType::String_, "this is a fun 'literal'"),
                Token::new(1, TokenType::Eof, ""),
            ],
        );
    }

    #[test]
    fn test_string_literal_unclosed_scan() {
        assert_scan(
            "\"a literal\n more",
            vec![
                Token::new(2, TokenType::ErrorUnclosedString, "a literal\n more"),
                Token::new(2, TokenType::Eof, ""),
            ],
        );
    }

    #[test]
    fn test_comment_scan() {
        assert_scan(
            "1 // comment \n 2",
            vec![
                Token::new(1, TokenType::Number, "1"),
                Token::new(1, TokenType::Comment, "// comment "),
                Token::new(2, TokenType::Number, "2"),
                Token::new(2, TokenType::Eof, ""),
            ],
        );
    }

    #[test]
    fn test_number_scan() {
        assert_scan(
            "0123",
            vec![
                Token::new(1, TokenType::Number, "0123"),
                Token::new(1, TokenType::Eof, ""),
            ],
        );

        assert_scan(
            "0123.456",
            vec![
                Token::new(1, TokenType::Number, "0123.456"),
                Token::new(1, TokenType::Eof, ""),
            ],
        );

        assert_scan(
            "0.4",
            vec![
                Token::new(1, TokenType::Number, "0.4"),
                Token::new(1, TokenType::Eof, ""),
            ],
        );

        assert_scan(
            "12.",
            vec![
                Token::new(1, TokenType::ErrorMalformedNumber, "12."),
                Token::new(1, TokenType::Eof, ""),
            ],
        );
    }
}
