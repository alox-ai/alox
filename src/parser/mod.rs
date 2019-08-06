use std::ops::Range;

use logos::{Logos, Source};
use logos::internal::LexerInternal;

use lexer::Lexer;
use lexer::Token;

use crate::ast::*;

pub mod lexer;

pub struct ParserError {
    location: Range<usize>,
    message: String,
}

impl ParserError {
    pub fn from<T>(lexer: &mut Lexer, message: &str) -> Result<T, Self> {
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

                let spaces = " ".repeat(self.location.start - index - (line_num - 1));
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

    let mut lexer = Lexer::new(code.as_str());
    while !lexer.has(Token::End) {
        println!("token: {:?} {}", lexer.token(), lexer.slice());

        if lexer.has(Token::Fun) {
            // function header
            lexer.advance();

            match parse_function(&mut lexer) {
                Ok(dec) => {
                    program.nodes.push(Node::Function(Box::new(dec)));
                }
                Err(err) => {
                    let err = err.to_string(lexer.inner.source);
                    eprintln!("{}", err);
                    return None;
                }
            }
        } else {
            lexer.advance();
        }
    }

    Some(program)
}

pub fn parse_function(lexer: &mut Lexer) -> Result<Function, ParserError> {
    // fun main
    lexer.expect(Token::Identifier, "Expected function name")?;
    let function_name = lexer.slice().to_string();
    lexer.advance();

    // fun main(
    lexer.skip(Token::LeftParen, "Expected opening paren")?;

    // fun main(x: a::a::a, y: b::b::b
    let mut arguments = vec![];
    if lexer.has(Token::Identifier) {
        while lexer.has(Token::Identifier) {
            let arg_name = lexer.slice().to_string();
            lexer.advance();
            lexer.skip(Token::Colon, "Expected parameter type")?;
            lexer.expect(Token::Identifier, "Expected type name after colon")?;
            let arg_type = parse_path_ident(lexer);
            match arg_type {
                Some(arg_type) => arguments.push((arg_name, arg_type)),
                None => return ParserError::from(lexer, "Expected type name after colon")
            }
            if lexer.has(Token::Comma) {
                lexer.advance();
            } else {
                break;
            }
        }

        // fun main(x: a::a::a, y: b::b::b)
        lexer.skip(Token::RightParen, "Expected closing paren")?;
    } else if lexer.has(Token::RightParen) {
        // fun main()
        lexer.advance();
    } else {
        return ParserError::from(lexer, "Unexpected symbol");
    }

    // fun main(x: a::a::a, y: b::b::b): c::c::c
    let return_type: (Path, String);
    if lexer.has(Token::Colon) {
        lexer.advance();
        lexer.expect(Token::Identifier, "Expected return type after colon")?;
        let ret_type_o = parse_path_ident(lexer);
        if let None = ret_type_o { return ParserError::from(lexer, "Expected return type after colon"); }
        return_type = ret_type_o.unwrap();
    } else {
        return_type = (Path(vec![]), "Void".to_string());
    }

    let mut statements = vec![];
    if lexer.has(Token::Semicolon) {
        lexer.advance();
    } else if lexer.has(Token::LeftBrace) {
        lexer.advance();
        while !lexer.has(Token::RightBrace) {
            let statement = parse_statement(lexer)?;
            dbg!(statement.clone());
            statements.push(statement);
        }
        lexer.advance();
    } else {
        lexer.unexpected()?;
    }

    Ok(Function {
        name: function_name,
        arguments,
        return_type,
        refinements: vec![],
        permissions: vec![],
        statements,
    })
}

pub fn parse_statement(lexer: &mut Lexer) -> Result<Statement, ParserError> {
    if lexer.has(Token::Return) {
        lexer.advance(); // skip return
        let expression = parse_expression(lexer)?;
        return Ok(Statement::Return(Box::new(Return { expression })));
    } else if lexer.has(Token::Let) {
        lexer.advance(); // skip let
        lexer.expect(Token::Identifier, "Expected variable name after 'let'");
        let name = lexer.slice().to_string();

        let type_name: Option<(Path, String)>;
        if lexer.has(Token::Colon) {
            lexer.advance();
            lexer.expect(Token::Identifier, "Expected type after colon")?;
            let ret_type_o = parse_path_ident(lexer);
            if let None = ret_type_o { return ParserError::from(lexer, "Couldn't parse type after colon"); }
            type_name = Some(ret_type_o.unwrap());
        } else {
            type_name = None;
        }
        lexer.advance();

        lexer.skip(Token::Equals, "Expected equal sign after variable declaration");

        let initial_expression = parse_expression(lexer)?;

        return Ok(Statement::VariableDeclaration(Box::new(VariableDeclaration {
            name,
            type_name,
            initial_expression,
        })));
    }
    println!("couldn't parse statement?");
    lexer.unexpected()
}

pub fn parse_expression(lexer: &mut Lexer) -> Result<Expression, ParserError> {
    if lexer.has(Token::IntegerLiteral) {
        let num = lexer.slice().parse::<i64>().unwrap();
        lexer.advance();
        return Ok(Expression::IntegerLiteral(Box::new(IntegerLiteral(num))));
    } else if lexer.has(Token::Identifier) {
        let variable_reference = VariableReference::from_str(lexer.slice());
        lexer.advance();
        return Ok(Expression::VariableReference(Box::new(variable_reference)));
    }
    lexer.advance();
    Ok(Expression::IntegerLiteral(Box::new(IntegerLiteral(1))))
}

pub fn parse_path_ident(lexer: &mut Lexer) -> Option<(Path, String)> {
    let mut path = Path(vec![]);
    while lexer.has(Token::Identifier) {
        let part = lexer.slice().to_string();
        lexer.advance();
        if lexer.has(Token::DoubleColon) {
            path = path.append(part);
            lexer.advance();
        } else {
            return Some((path, part));
        }
    }
    None
}