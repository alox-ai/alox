use core::borrow::{Borrow, BorrowMut};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard};

use crate::ir::*;

pub struct Printer {
    depth: usize
}

impl Printer {
    pub fn new() -> Self {
        Self {
            depth: 0
        }
    }

    pub fn push(&mut self) {
        self.depth += 1;
    }

    pub fn pop(&mut self) {
        self.depth -= 1;
    }

    pub fn print(&self, s: String) {
        if self.depth > 0 {
            print!("{}", "  ".repeat(self.depth));
        }
        println!("{}", s);
    }

    pub fn print_module(&mut self, module: &Module) {
        self.print(format!("; Module: {}::{}", module.path.to_string(), module.name));
        for declaration in module.declarations.iter() {
            self.print_declaration(declaration);
        }
    }

    pub fn print_declaration(&mut self, dec: &Arc<Declaration>) {
        match **dec {
            Declaration::Function(ref function) => {
                self.print_function(function);
            }
            _ => {}
        }
    }

    pub fn print_function(&mut self, function: &Box<Function>) {
        self.print(format!("def @{}:", function.name));

        let mut id = 0;
        self.push();
        for block in function.blocks.iter() {
            self.print_block(id, block);
            id += 1;
        }
        self.pop();
    }

    pub fn print_block(&mut self, id: usize, mut block: &Arc<Mutex<Block>>) {
        let mut block = block.lock().unwrap();
        self.print(format!("block#{}:", id));

        let mut id = 0;
        self.push();
        let mut map: HashMap<*const Mutex<Instruction>, usize> = HashMap::new();
        for instruction in block.instructions.iter() {
            map.insert(instruction.as_ref() as *const Mutex<Instruction>, id);
            self.print_instruction(&map, id, instruction);
            id += 1;
        }
        self.pop();
    }

    pub fn print_instruction(&mut self, map: &HashMap<*const Mutex<Instruction>, usize>, id: usize, instruction: &Arc<Mutex<Instruction>>) {
        let mut instruction = instruction.lock().unwrap();
        match *instruction {
            Instruction::DeclarationReference(ref d) => {
                let (path, name) = &d.name;
                let path_name = match path {
                    Some(path) => path.to_string(),
                    None => "".to_string()
                };
                let filled = match *(d.declaration.lock().unwrap()) {
                    None => "*",
                    Some(_) => ""
                };

                self.print(format!("%{} = @{}::{}{}", id, path_name, name, filled))
            }
            Instruction::IntegerLiteral(ref i) => {
                self.print(format!("%{} = {}", id, i.as_ref().0))
            }
            Instruction::FunctionCall(ref call) => {
                let function = call.function.as_ref() as *const Mutex<Instruction>;
                let function_id = if let Some(id) = map.get(&function) { *id } else { 99999 };

                let mut arg_ids = Vec::with_capacity(call.arguments.len());
                for arg in call.arguments.iter() {
                    let p = arg.as_ref() as *const Mutex<Instruction>;
                    let arg_id = if let Some(id) = map.get(&p) { *id } else { 88888 };
                    arg_ids.push(arg_id);
                }
                let arg_strings: Vec<String> = arg_ids.iter().map(|i| format!("%{}", *i)).collect();
                let args_connected = arg_strings.join(", ");

                self.print(format!("%{} = %{}({})", id, function_id, args_connected))
            }
            Instruction::Return(ref ret) => {
                let value = ret.instruction.as_ref() as *const Mutex<Instruction>;
                let value_id = if let Some(id) = map.get(&value) { *id } else { 77777 };

                self.print(format!("ret %{}", value_id))
            }
            _ => {
                self.print(format!("%{} = unprintable", id))
            }
        }
    }
}
