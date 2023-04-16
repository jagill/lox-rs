use super::TokenType;

#[derive(Debug, PartialEq, Eq)]
pub struct Token<'a> {
    pub line: usize,
    pub typ: TokenType,
    pub lexeme: &'a str,
}

impl<'a> Token<'a> {
    pub fn new(line: usize, typ: TokenType, lexeme: &'a str) -> Self {
        Self { line, typ, lexeme }
    }
}
