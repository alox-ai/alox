use codespan_reporting::diagnostic::{Diagnostic, Severity};

use crate::ir::*;
use crate::ir::pass::Pass;

pub struct SemanticAnalysisPass;

impl Pass for SemanticAnalysisPass {
    fn pass_declaration(&self, compiler: &Compiler, dec: &mut Declaration) {
        match *dec {
            Declaration::Function(ref mut function) => {}
            Declaration::Struct(ref mut struc) => {
                for function in struc.functions.iter_mut() {
                    match function {
                        Declaration::Function(function) => {
                            if struc.kind == StructKind::Struct && function.is_behaviour() {
                                let diagnostic = Diagnostic::new(Severity::Error)
                                    .with_message("behaviours are only allowed in actors");
                                // TODO: add label
                                compiler.add_diagnostic(diagnostic);
                            }
                        }
                        _ => {}
                    }
                }
            }
            Declaration::Trait(_) => {}
            Declaration::Variable(_) => {}
            Declaration::Type(_) => {}
        }
    }
}
