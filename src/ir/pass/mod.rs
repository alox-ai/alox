use crate::ir::{Declaration, Function, Module, Instruction};

pub trait Pass {
    fn pass(&self, module: &mut Module) {
        for dec in module.declarations.iter_mut() {
            self.pass_declaration(dec);
        }
    }

    fn pass_declaration(&self, dec: &mut Declaration) {
        match *dec {
            Declaration::Function(ref mut function) => {
                self.pass_function(function);
            }
            Declaration::Behaviour(_) => {}
            Declaration::Actor(ref mut actor) => {
                for function in actor.functions.iter_mut() {
                    self.pass_declaration(function);
                }

                for behaviour in actor.behaviours.iter_mut() {
                    self.pass_declaration(behaviour);
                }
            }
            Declaration::Struct(ref mut struc) => {
                for function in struc.functions.iter_mut() {
                    self.pass_declaration(function);
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
        'blocks: for (block_id, _) in function.blocks.iter().enumerate() {
            // compare every block to every other block
            'other_blocks: for (other_block_id, other_block) in function.blocks.iter().enumerate() {
                // make sure we're not comparing the same block
                if other_block_id == block_id { continue 'other_blocks; }

                // check if block is referenced in other_block
                for instruction in other_block.instructions.iter() {
                    match *instruction {
                        Instruction::Jump(ref j) => {
                            let referred_block_id = j.block.0 as usize;
                            if block_id == referred_block_id {
                                continue 'blocks;
                            }
                        }
                        Instruction::Branch(ref b) => {
                            let referred_true_block_id = b.true_block.0 as usize;
                            let referred_false_block_id = b.false_block.0 as usize;
                            if block_id == referred_true_block_id || block_id == referred_false_block_id {
                                continue 'blocks;
                            }
                        }
                        _ => {}
                    }
                }
            }
            // block isn't being used
            dead_blocks.push(block_id);
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