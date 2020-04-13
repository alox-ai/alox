use crate::ir::*;
use crate::ir::pass::deadbranch::DeadBranchRemovalPass;
use crate::ir::pass::semantics::SemanticAnalysisPass;

pub mod semantics;
pub mod deadbranch;

pub struct PassManager {
    passes: Vec<Box<dyn Pass>>,
}

impl PassManager {
    pub fn new() -> Self {
        Self {
            passes: vec![
                Box::new(SemanticAnalysisPass),
            ],
        }
    }

    pub fn optimize() -> Self {
        Self {
            passes: vec![
                Box::new(DeadBranchRemovalPass),
                Box::new(SemanticAnalysisPass),
            ],
        }
    }

    pub fn apply(&self, compiler: &Compiler) {
        let mut modules = compiler.modules.write().unwrap();
        for module in modules.iter_mut() {
            for pass in self.passes.iter() {
                pass.pass(compiler, module);
            }
        }
    }
}

pub trait Pass {
    fn pass(&self, compiler: &Compiler, module: &mut Module) {
        for dec in module.declarations.iter_mut() {
            self.pass_declaration(compiler, dec);
        }
    }

    fn pass_declaration(&self, compiler: &Compiler, dec: &mut Declaration) {
        match *dec {
            Declaration::Function(ref mut function) => {
                self.pass_blocks(compiler, &mut function.blocks);
            }
            Declaration::Struct(ref mut struc) => {
                for function in struc.functions.iter_mut() {
                    self.pass_declaration(compiler, function);
                }
            }
            Declaration::Trait(_) => {}
            Declaration::Variable(_) => {}
            Declaration::Type(_) => {}
        }
    }

    fn pass_blocks(&self, compiler: &Compiler, blocks: &mut Vec<Block>) {}
}
