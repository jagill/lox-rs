use super::{Token, TokenType};
use TokenType::*;

pub struct Scanner<'a> {
    source: &'a str,
    char_idxs: std::iter::Peekable<std::str::CharIndices<'a>>,
    start: usize,
    current: usize,
    line: usize,
}

impl<'a> Scanner<'a> {
    pub fn new(source: &'a str) -> Self {
        Scanner {
            source,
            char_idxs: source.char_indices().peekable(),
            start: 0,
            current: 0,
            line: 1,
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
                    while self.peek() != Some('\n') {
                        self.advance();
                    }
                    Comment
                } else {
                    Slash
                }
            }

            '"' => {
                while let Some(ch) = self.peek() {
                    self.advance();
                    match ch {
                        '"' => {
                            break;
                        }
                        '\n' => {
                            self.line += 1;
                        }
                        _ => (),
                    }
                }
                String_
            }

            '0'..='9' => {
                self.advance_number();
                Number
            }

            'A'..='Z' | 'a'..='z' | '_' => {
                while let Some(ch) = self.peek() {
                    if is_kw_char(ch) {
                        self.advance();
                    } else {
                        break;
                    }
                }
                TokenType::get(self.current_lexeme())
            }

            _ => ErrorUnknownToken,
        };

        let lexeme = self.current_lexeme();
        Some(Token::new(self.line, typ, lexeme))
    }

    fn skip_whitespace(&mut self) {
        let whitespace_chars = [' ', '\n', '\t', '\r'];
        let mut did_advance = false;
        while let Some((_, ch)) = self
            .char_idxs
            .next_if(|(_, c)| whitespace_chars.contains(c))
        {
            if ch == '\n' {
                self.line += 1;
            }
            did_advance = true;
        }
        if did_advance {
            self.current = self.next_idx();
        }
    }

    fn current_lexeme(&self) -> &'a str {
        &self.source[self.start..self.current]
    }

    // Advance past the rest of the number. It assumes the first digit has already been consumed.
    fn advance_number(&mut self) {
        while let Some(ch) = self.peek() {
            match ch {
                '0'..='9' => {
                    self.advance();
                }
                '.' => {
                    // TODO: Handle this
                    break;
                }
                _ => break,
            }
        }
    }

    // Advance to the next char, if any
    fn advance(&mut self) -> Option<char> {
        let (_idx, ch) = self.char_idxs.next()?;
        self.current = self.next_idx();
        Some(ch)
    }

    fn next_idx(&mut self) -> usize {
        self.char_idxs.peek().map_or(self.source.len(), |(i, _)| *i)
    }

    // If the next char is equal to `ch`, advance and return true.  Else, return false.
    fn match_next(&mut self, ch: char) -> bool {
        if self.peek() == Some(ch) {
            self.advance();
            true
        } else {
            false
        }
    }

    // Return the next char without advancing.
    fn peek(&mut self) -> Option<char> {
        self.char_idxs.peek().map(|(_i, c)| *c)
    }
}

impl<'a> Iterator for Scanner<'a> {
    type Item = Token<'a>;

    fn next(&mut self) -> Option<Token<'a>> {
        self.scan_token()
    }
}

fn is_kw_char(ch: char) -> bool {
    ('A' <= ch && ch <= 'Z') || ('a' <= ch && ch <= 'z') || ('0' <= ch && ch <= '9') || ch == '_'
}
