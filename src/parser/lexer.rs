use logos::Logos;

#[derive(Logos, Debug, PartialEq)]
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
