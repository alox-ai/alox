use std::ops::Range;

use codespan_reporting::diagnostic::{Diagnostic, Label};
use lalrpop_util::*;
use lalrpop_util::lexer::Token;

use crate::ast::*;
use crate::diagnostic::*;

lalrpop_mod!(#[allow(clippy::all)] #[allow(warnings)] #[allow(unknown_lints)] pub grammar, "/parser/grammar.rs");

pub struct Parser {
    diagnostics: DiagnosticManager,
}

impl Parser {
    pub fn new() -> Self {
        Self {
            diagnostics: DiagnosticManager::new()
        }
    }

    fn get_range(error: ParseError<usize, Token, &str>) -> Range<usize> {
        match error {
            ParseError::InvalidToken { location } => (location..location),
            ParseError::UnrecognizedEOF { location, expected: _ } => (location..location),
            ParseError::UnrecognizedToken { token, expected: _ } => (token.0..token.2),
            ParseError::ExtraToken { token } => (token.0..token.2),
            ParseError::User { error: _ } => (0..0)
        }
    }

    fn add_parse_error(&mut self, file_id: FileId, error: ParseError<usize, Token, &str>) {
        let message = match &error {
            ParseError::InvalidToken { location: _ } => "encountered invalid token while parsing".to_string(),
            ParseError::UnrecognizedEOF { location: _, expected: _ } => {
                format!("encountered unexpected EOF while parsing")
            }
            ParseError::UnrecognizedToken { token, expected: _ } => {
                format!("encountered unexpected '{}' while parsing", (token.1).1)
            }
            ParseError::ExtraToken { token } => {
                format!("encountered unexpected '{}' while parsing but it is not needed", (token.1).1)
            }
            ParseError::User { error } => {
                error.to_string()
            }
        };
        let label_message = match &error {
            ParseError::InvalidToken { location: _ } => Some("this token is invalid"),
            ParseError::UnrecognizedEOF { location: _, expected: _ } => Some("unexpected end of file"),
            ParseError::UnrecognizedToken { token: _, expected: _ } => Some("unexpected token"),
            ParseError::ExtraToken { token: _ } => Some("unexpected token"),
            ParseError::User { error } => Some(*error),
        };
        let range = Self::get_range(error);

        let mut label = Label::primary(file_id, range);
        if let Some(label_message) = label_message {
            label = label.with_message(label_message);
        }

        let diagnostic = Diagnostic::error()
            .with_message(message)
            .with_labels(vec![label]);
        self.diagnostics.add_diagnostic(diagnostic);
    }

    pub fn parse(&mut self, path: Path, file_name: String, code: String) -> Option<Program> {
        let file_id = self.diagnostics.add_file(file_name.clone(), code.clone());
        let file_name_parts: Vec<&str> = file_name.rsplitn(2, ".").collect();
        let module_name = file_name_parts.get(0).unwrap();

        let mut errors: Vec<ErrorRecovery<usize, Token, &str>> = Vec::new();
        let result: Result<(Vec<Path>, Vec<Node>), ParseError<usize, Token, &str>> = grammar::ProgramParser::new().parse(&mut errors, &code);

        if errors.len() > 0 {
            for error in errors {
                self.add_parse_error(file_id, error.error);
            }
            return None;
        }

        return match result {
            Ok((imports, nodes)) => {
                Some(Program {
                    path,
                    file_name: module_name.to_string(),
                    imports,
                    nodes,
                })
            }
            Err(error) => {
                self.add_parse_error(file_id, error);
                None
            }
        };
    }

    pub fn emit_errors(&self) {
        self.diagnostics.emit_errors();
    }
}

