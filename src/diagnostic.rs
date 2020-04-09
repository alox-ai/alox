use std::collections::HashMap;

use codespan_reporting::diagnostic::Diagnostic;
use codespan_reporting::files::SimpleFiles;
use codespan_reporting::term::termcolor::{ColorChoice, StandardStream};

pub type FileId = usize;

pub struct DiagnosticManager {
    files: SimpleFiles<String, String>,
    file_ids: HashMap<String, FileId>,
    messages: Vec<Diagnostic<FileId>>,
}

impl DiagnosticManager {
    pub fn new() -> Self {
        Self {
            files: SimpleFiles::new(),
            file_ids: HashMap::new(),
            messages: Vec::new(),
        }
    }

    pub fn add_file(&mut self, name: String, source: String) -> FileId {
        let id = self.files.add(name.clone(), source);
        self.file_ids.insert(name, id);
        id
    }

    pub fn get_file_id<T>(&self, file_name: T) -> Option<FileId> where T: AsRef<str> {
        self.file_ids.get(file_name.as_ref()).map(|e| *e)
    }

    pub fn add_diagnostic(&mut self, diagnostic: Diagnostic<FileId>) {
        self.messages.push(diagnostic);
    }

    pub fn emit_errors(&self) {
        let writer = StandardStream::stderr(ColorChoice::Always);
        let config = codespan_reporting::term::Config::default();
        for message in self.messages.iter() {
            codespan_reporting::term::emit(&mut writer.lock(), &config, &self.files, message).unwrap()
        }
    }
}