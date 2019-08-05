use std::ops::Range;

use logos::Logos;

#[derive(Logos, Debug, PartialEq, Clone)]
pub enum Token {
    // required by Logos
    #[end] End,
    #[error] Error,

    // keywords
    #[token = "let"] Let,
    #[token = "var"] Var,
    #[token = "fun"] Fun,
    #[token = "def"] Def,

    // symbols
    #[token = "->"] ThinArrow,
    #[token = "=>"] ThickArrow,
    #[token = "::"] DoubleColon,
    #[token = "("] LeftParen,
    #[token = ")"] RightParen,
    #[token = "["] LeftBracket,
    #[token = "]"] RightBracket,
    #[token = "{"] LeftBrace,
    #[token = "}"] RightBrace,
    #[token = "<"] LeftAngle,
    #[token = ">"] RightAngle,
    #[token = "."] Period,
    #[token = "="] Equals,
    #[token = "-"] Minus,
    #[token = "+"] Plus,
    #[token = "*"] Star,
    #[token = "/"] Slash,
    #[token = ","] Comma,
    #[token = ":"] Colon,

    #[regex = "[0-9]+"] IntegerLiteral,
    #[regex = "[a-zA-Z][a-zA-Z0-9_]*"] Identifier,
}

pub struct Lexer<'a> {
    pub inner: logos::Lexer<Token, &'a str>,
    buffer: Vec<Token>,
    past_ranges: Vec<Range<usize>>,
}

impl<'a> Lexer<'a> {
    pub fn new(code: &'a str) -> Self {
        Self {
            inner: Token::lexer(code),
            buffer: Vec::with_capacity(3),
            past_ranges: Vec::with_capacity(3),
        }
    }

    pub fn advance(&mut self) {
        if self.past_ranges.len() > 0 {
            self.past_ranges.remove(0);
        }
        if self.buffer.len() > 0 {
            self.buffer.remove(0);
        } else {
            self.inner.advance();
            self.past_ranges.push(self.inner.range());
        }
    }

    pub fn token(&self) -> &Token {
        if self.buffer.len() > 0 {
            self.buffer.get(0).unwrap()
        } else {
            &self.inner.token
        }
    }

    pub fn slice(&self) -> &str {
        unsafe { self.inner.source.get_unchecked(self.range()) }
    }

    pub fn range(&self) -> Range<usize> {
        if self.buffer.len() > 0 {
            self.past_ranges.get(0).unwrap().clone()
        } else {
            self.inner.range()
        }
    }

    pub fn peek(&mut self, ahead: usize) -> &Token {
        assert!(ahead > 0, "use Lexer::token() instead");
        if self.buffer.len() > ahead {
            self.buffer.get(ahead).unwrap()
        } else {
            self.buffer.push(self.inner.token.clone());
            self.inner.advance();
            self.past_ranges.push(self.inner.range());
            self.buffer.get(ahead - 1).unwrap()
        }
    }
}
