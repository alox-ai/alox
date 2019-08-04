use logos::{Lexer, Logos, Source};

use lexer::Token;

use crate::ast::*;

pub mod lexer;

pub fn parse_function_declaration(lexer: &mut Lexer<Token, &str>) -> Option<FunctionDeclaration> {

    // fun main
    if lexer.token != Token::Identifier { return None; }
    let function_name = lexer.slice().to_string();
    lexer.advance();

    // fun main(
    if lexer.token != Token::LeftParen { return None; }
    lexer.advance();

    // fun main(x: a::a::a, y: b::b::b
    let mut args = vec![];
    if lexer.token == Token::Identifier {
        while lexer.token == Token::Identifier {
            let arg_name = lexer.slice().to_string();
            lexer.advance();
            if lexer.token != Token::Colon { return None; }
            lexer.advance();
            if lexer.token != Token::Identifier { return None; }
            let arg_type = parse_path_ident(lexer);
            if let None = arg_type { return None; }
            if let Some(arg_type) = arg_type {
                args.push((arg_name, arg_type));
            }
            if lexer.token == Token::Comma {
                lexer.advance();
            } else {
                break;
            }
        }

        // fun main(x: a::a::a, y: b::b::b)
        if lexer.token != Token::RightParen { return None; }
        lexer.advance();
    } else if lexer.token == Token::RightParen {
        // fun main()
        lexer.advance();
    } else {
        return None;
    }


    // fun main(x: a::a::a, y: b::b::b): c::c::c
    let ret_type: (Path, String);
    if lexer.token == Token::Colon {
        lexer.advance();
        if lexer.token != Token::Identifier { return None; }
        let ret_type_o = parse_path_ident(lexer);
        if let None = ret_type_o { return None; }
        ret_type = ret_type_o.unwrap();
    } else {
        ret_type = (Path::of(""), "Void".to_string());
    }

    Some(FunctionDeclaration {
        name: function_name,
        arguments: args,
        return_type: ret_type,
        refinements: vec![],
        permissions: vec![],
    })
}

pub fn parse<'a>(file: String) {
    let mut lexer: Lexer<Token, &str> = Token::lexer(file.as_str());
    while lexer.token != Token::End {
        println!("token: {:?} {}", lexer.token, lexer.slice());

        if lexer.token == Token::Fun {
            // function header
            lexer.advance();
            if let Some(dec) = parse_function_declaration(&mut lexer) {
                dbg!(dec);
            }
        } else {
            lexer.advance();
        }
    }
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