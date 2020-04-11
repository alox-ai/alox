use std::collections::HashMap;
use std::io::Error;

use codespan_reporting::diagnostic::{Diagnostic, Severity};
use codespan_reporting::files::SimpleFiles;
use codespan_reporting::term::{Chars, Config, DisplayStyle, Styles};
use codespan_reporting::term::termcolor::{ColorChoice, StandardStream};

pub type FileId = usize;

const ASCII_CHARS: Chars = Chars {
    source_border_top_left: '/',
    source_border_top: '-',
    source_border_left: '|',
    source_border_left_break: '*',
    note_bullet: '=',
    single_primary_caret: '^',
    single_secondary_caret: '-',
    multi_primary_caret_start: '^',
    multi_primary_caret_end: '^',
    multi_secondary_caret_start: '\'',
    multi_secondary_caret_end: '\'',
    multi_top_left: '/',
    multi_top: '-',
    multi_bottom_left: '\\',
    multi_bottom: '-',
    multi_left: '|',
};

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

    pub fn has_errors(&self) -> bool {
        for message in self.messages.iter() {
            if message.severity == Severity::Error {
                return true;
            }
        }
        false
    }

    pub fn emit_to_string(&self) -> String {
        let config = Config {
            display_style: DisplayStyle::Short,
            tab_width: 4,
            styles: Styles::default(),
            chars: ASCII_CHARS,
        };
        let mut writer = StringBuf::default();
        for message in self.messages.iter() {
            codespan_reporting::term::emit(&mut writer, &config, &self.files, message).unwrap()
        }
        writer.buffer
    }

    pub fn emit_errors(&self) {
        let writer = StandardStream::stderr(ColorChoice::Always);
        let config = codespan_reporting::term::Config::default();
        for message in self.messages.iter() {
            codespan_reporting::term::emit(&mut writer.lock(), &config, &self.files, message).unwrap()
        }
    }
}

// This is a big workaround for codespan only writing to a termcolor WriteColor.

struct StringBuf {
    buffer: String,
}

impl Default for StringBuf {
    fn default() -> Self {
        Self { buffer: String::new() }
    }
}

impl std::io::Write for StringBuf {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        for b in buf {
            self.buffer.push(*b as char);
        }
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> { Ok(()) }
}

impl codespan_reporting::term::termcolor::WriteColor for StringBuf {
    fn supports_color(&self) -> bool {
        false
    }

    fn set_color(&mut self, _: &codespan_reporting::term::termcolor::ColorSpec) -> std::io::Result<()> {
        Ok(())
    }

    fn reset(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
