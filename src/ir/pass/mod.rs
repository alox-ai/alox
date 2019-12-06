use std::sync::{Mutex, Arc, RwLock};

use crate::ir::{Block, Declaration, Function, Module, Instruction};

pub trait Pass {
    fn pass(&self, module: &Module) {
        for dec in module.declarations.iter() {
            self.pass_declaration(dec);
        }
    }

    fn pass_declaration(&self, dec: &Arc<RwLock<Declaration>>) {
        match *dec.write().unwrap() {
            Declaration::Function(ref mut function) => {
                self.pass_function(function);
            }
            Declaration::Behaviour(_) => {}
            Declaration::Actor(ref mut actor) => {
                for function in actor.functions.write().unwrap().iter() {
                    if let Some(ref func) = *function.0.lock().unwrap() {
                        self.pass_declaration(func);
                    }
                }

                for behaviour in actor.behaviours.write().unwrap().iter() {
                    if let Some(ref behave) = *behaviour.0.lock().unwrap() {
                        self.pass_declaration(behave);
                    }
                }
            }
            Declaration::Struct(ref mut struc) => {
                for function in struc.functions.write().unwrap().iter() {
                    if let Some(ref func) = *function.0.lock().unwrap() {
                        self.pass_declaration(func);
                    }
                }
            }
            Declaration::Trait(_) => {}
            Declaration::Variable(_) => {}
            Declaration::Type(_) => {}
        }
    }
    fn pass_function(&self, function: &mut Box<Function>);
}

pub struct DeadBranchRemovalPass {}

impl Pass for DeadBranchRemovalPass {
    // TODO: account for dead blocks that refer to each other
    fn pass_function(&self, function: &mut Box<Function>) {
        let mut dead_blocks = vec![];
        // go over every block
        'blocks: for (i, block) in function.blocks.iter().enumerate() {
            let block_ptr = block.as_ref() as *const Mutex<Block>;
            // compare every block to every other block
            'other_blocks: for other_block in function.blocks.iter() {
                let other_block_ptr = other_block.as_ref() as *const Mutex<Block>;
                // make sure we're not comparing two blocks
                if other_block_ptr == block_ptr { continue 'other_blocks; }

                // check if block is referenced in other_block
                for instruction in other_block.lock().unwrap().instructions.iter() {
                    match *instruction.lock().unwrap() {
                        Instruction::Jump(ref j) => {
                            let referred_block_ptr = j.block.as_ref() as *const Mutex<Block>;
                            if block_ptr == referred_block_ptr {
                                continue 'blocks;
                            }
                        }
                        Instruction::Branch(ref b) => {
                            let referred_true_block_ptr = b.true_block.as_ref() as *const Mutex<Block>;
                            let referred_false_block_ptr = b.false_block.as_ref() as *const Mutex<Block>;
                            if block_ptr == referred_true_block_ptr || block_ptr == referred_false_block_ptr {
                                continue 'blocks;
                            }
                        }
                        _ => {}
                    }
                }
            }
            // block isn't being used
            dead_blocks.push(i);
        }
        // ordering and reversing the indexes means we don't
        // have to do any index math when a block is removed
        dead_blocks.sort();
        dead_blocks.reverse();
        for i in dead_blocks {
            if i != 0 { // we don't want to remove the first block
                function.blocks.remove(i);
            }
        }
    }
}