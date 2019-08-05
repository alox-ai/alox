use std::ops::Range;

use logos::{Lexer, Logos, Source};
use logos::internal::LexerInternal;

use lexer::Token;

use crate::ast::*;

pub mod lexer;

pub struct ParserError {
    location: Range<usize>,
    message: String,
}

impl ParserError {
    pub fn from<T>(lexer: &mut Lexer<Token, &str>, message: &str) -> Result<T, Self> {
        Err(Self {
            location: lexer.range(),
            message: message.to_string(),
        })
    }

    pub fn to_string(&self, source: &str) -> String {
        let mut buffer = format!("error: {}\n", self.message);

        let len = source.len();
        let mut line_num = 0;
        let mut index = 0;
        for line in source.lines() {
            line_num += 1;
            if self.location.start > index && self.location.end < index + line.len() {
                // we found the line
                let err_header = format!("{} | ", line_num);
                let blank_header = format!("{} | ", " ".repeat(line_num.to_string().len()));
                buffer.push_str(&blank_header);
                buffer.push_str(&format!("\n{}{}\n", err_header, line));

                let spaces = " ".repeat(self.location.start - index);
                let arrows = "^".repeat(self.location.end - self.location.start);

                buffer.push_str(&format!("{}{}{}\n", blank_header, spaces, arrows));
            }
            index += line.len();
        }
        buffer
    }
}

pub fn parse<'a>(path: Path, module_name: String, code: String) -> Option<Program> {
    let mut program = Program {
        path,
        file_name: module_name,
        imports: vec![],
        nodes: vec![],
    };

    let mut lexer: Lexer<Token, &str> = Token::lexer(code.as_str());
    while lexer.token != Token::End {
        println!("token: {:?} {}", lexer.token, lexer.slice());

        if lexer.token == Token::Fun {
            // function header
            lexer.advance();

            match parse_function(&mut lexer) {
                Ok(dec) => {
                    program.nodes.push(Node::Function(Box::new(dec)));
                }
                Err(err) => {
                    let err = err.to_string(lexer.source);
                    eprintln!("{}", err);
                    return None;
                }
            }
        } else if lexer.token == Token::Def {
            // function definition
            lexer.advance();
        } else {
            lexer.advance();
        }
    }

    Some(program)
}

pub fn parse_function(lexer: &mut Lexer<Token, &str>) -> Result<Function, ParserError> {
    // fun main
    if lexer.token != Token::Identifier { return ParserError::from(lexer, "Expected function name"); }
    let function_name = lexer.slice().to_string();
    lexer.advance();

    // fun main(
    if lexer.token != Token::LeftParen { return ParserError::from(lexer, "Expected opening paren"); }
    lexer.advance();

    // fun main(x: a::a::a, y: b::b::b
    let mut arguments = vec![];
    if lexer.token == Token::Identifier {
        while lexer.token == Token::Identifier {
            let arg_name = lexer.slice().to_string();
            lexer.advance();
            if lexer.token != Token::Colon { return ParserError::from(lexer, "Expected parameter type"); }
            lexer.advance();
            if lexer.token != Token::Identifier { return ParserError::from(lexer, "Expected type name after colon"); }
            let arg_type = parse_path_ident(lexer);
            match arg_type {
                Some(arg_type) => arguments.push((arg_name, arg_type)),
                None => return ParserError::from(lexer, "Expected type name after colon")
            }
            if lexer.token == Token::Comma {
                lexer.advance();
            } else {
                break;
            }
        }

        // fun main(x: a::a::a, y: b::b::b)
        if lexer.token != Token::RightParen {
            return ParserError::from(lexer, "Expected closing paren");
        }
        lexer.advance();
    } else if lexer.token == Token::RightParen {
        // fun main()
        lexer.advance();
    } else {
        return ParserError::from(lexer, "Unexpected symbol");
    }

    // fun main(x: a::a::a, y: b::b::b): c::c::c
    let return_type: (Path, String);
    if lexer.token == Token::Colon {
        lexer.advance();
        if lexer.token != Token::Identifier { return ParserError::from(lexer, "Expected return type after colon"); }
        let ret_type_o = parse_path_ident(lexer);
        if let None = ret_type_o { return ParserError::from(lexer, "Expected return type after colon"); }
        return_type = ret_type_o.unwrap();
    } else {
        return_type = (Path::of(""), "Void".to_string());
    }

    Ok(Function {
        name: function_name,
        arguments,
        return_type,
        refinements: vec![],
        permissions: vec![],
        statements: vec![],
    })
}

pub fn parse_path_ident(lexer: &mut Lexer<Token, &str>) -> Option<(Path, String)> {
    let mut path = Path(vec![]);
    while lexer.token == Token::Identifier {
        let part = lexer.slice().to_string();
        lexer.advance();
        if lexer.token == Token::DoubleColon {
            path = path.append(part);
            lexer.advance();
        } else {
            return Some((path, part));
        }
    }
    None
}