use super::TokenType;

#[derive(Debug)]
pub struct Token<'a> {
    pub(crate) line: usize,
    pub(crate) typ: TokenType,
    pub(crate) lexeme: &'a str,
}

impl<'a> Token<'a> {
    pub fn new(line: usize, typ: TokenType, lexeme: &'a str) -> Self {
        Self { line, typ, lexeme }
    }

    pub fn line(&self) -> usize {
        self.line
    }

    pub fn typ(&self) -> TokenType {
        self.typ
    }

    pub fn lexeme(&self) -> &str {
        self.lexeme
    }
}
