use core::borrow::{Borrow, BorrowMut};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard};

use crate::ir::*;
use std::env::var;

pub struct Printer {
    depth: usize,
}

impl Printer {
    pub fn new() -> Self {
        Self { depth: 0 }
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
        self.print(format!(
            "; Module: {}::{}",
            module.path.to_string(),
            module.name
        ));
        for declaration in module.declarations.iter() {
            self.print_declaration(declaration);
        }
    }

    pub fn print_declaration(&mut self, dec: &Arc<Declaration>) {
        match **dec {
            Declaration::FunctionHeader(ref header) => {
                self.print_function_header(header);
            }
            Declaration::Function(ref function) => {
                self.print_function(function);
            }
            Declaration::Variable(ref variable) => {
                self.print_variable(variable);
            }
            _ => {}
        }
    }

    pub fn print_variable(&mut self, variable: &Box<Variable>) {
        self.print(format!("let {}", variable.name));
    }

    pub fn print_function_header(&mut self, header: &Box<FunctionHeader>) {
        let mut joined_args = "".to_string();
        for (id, (arg, dec)) in (&header.arguments).iter().enumerate() {
            joined_args.push_str(&format!("%{}: {}", arg, dec.name()));
            if id < header.arguments.len() - 1 {
                joined_args.push_str(", ");
            }
        }

        let return_type_name = &header.return_type.name();
        self.print(format!(
            "fun @{}({}) -> {}:",
            header.name, joined_args, return_type_name
        ));
        self.push();
        self.print(format!("perms: {:?}", header.permissions));

        for (name, blocks) in header.refinements.iter() {
            self.print(format!("refinement %{}:", name));
            self.push();
            for (id, block) in blocks.iter().enumerate() {
                self.print_block(id, block);
            }
            self.pop();
        }

        self.pop();
    }

    pub fn print_function(&mut self, function: &Box<Function>) {
        let mut joined_args = "".to_string();
        let header = function.get_header();
        for (id, arg) in (&function.arguments).iter().enumerate() {
            let mut arg_str = format!("%{}", arg.0);

            // get the type name from the header declaration
            if let Some(header) = header.clone() {
                if let Declaration::FunctionHeader(ref header) = *header {
                    if let Some(header_arg) = header.arguments.get(id) {
                        arg_str = format!("%{}: {}", arg.0, &header_arg.1.name());
                    }
                }
            }

            joined_args.push_str(&arg_str);
            if id < function.arguments.len() - 1 {
                joined_args.push_str(", ");
            }
        }

        // get the return type name from the header declaration
        if let Some(header) = header.clone() {
            if let Declaration::FunctionHeader(ref header) = *header {
                let return_type_name = &header.return_type.name();
                self.print(format!(
                    "let @{} = ({}) -> {}:",
                    function.name, joined_args, return_type_name
                ));
            } else {
                self.print(format!("let @{} = ({}):", function.name, joined_args));
                self.print(format!("; Error: pointer isn't a header declaration!!"))
            }
        } else {
            self.print(format!("let @{}({}):", function.name, joined_args));
            self.print(format!("; Error: missing pointer to header declaration!!"))
        }

        self.push();
        for (id, block) in function.blocks.iter().enumerate() {
            self.print_block(id, block);
        }
        self.pop();
    }

    pub fn print_block(&mut self, id: usize, mut block: &Arc<Mutex<Block>>) {
        let mut block = block.lock().unwrap();
        self.print(format!("block#{}:", id));

        self.push();
        let mut map: HashMap<*const Mutex<Instruction>, usize> = HashMap::new();
        for (id, instruction) in block.instructions.iter().enumerate() {
            map.insert(instruction.as_ref() as *const Mutex<Instruction>, id);
            self.print_instruction(&map, id, instruction);
        }
        self.pop();
    }

    pub fn print_instruction(
        &mut self,
        map: &HashMap<*const Mutex<Instruction>, usize>,
        id: usize,
        instruction: &Arc<Mutex<Instruction>>,
    ) {
        let mut instruction = instruction.lock().unwrap();
        match *instruction {
            Instruction::DeclarationReference(ref d) => {
                let (path, name) = &d.name;
                let path_name = match path {
                    Some(path) => path.to_string(),
                    None => "".to_string(),
                };
                let filled = match *(d.declaration.0.lock().unwrap()) {
                    None => "*",
                    Some(_) => "",
                };

                self.print(format!("%{} = @{}::{}{}", id, path_name, name, filled))
            }
            Instruction::IntegerLiteral(ref i) => self.print(format!("%{} = {}", id, i.as_ref().0)),
            Instruction::FunctionCall(ref call) => {
                let function = call.function.as_ref() as *const Mutex<Instruction>;
                let function_id = if let Some(id) = map.get(&function) {
                    *id
                } else {
                    99999
                };

                let mut arg_ids = Vec::with_capacity(call.arguments.len());
                for arg in call.arguments.iter() {
                    let p = arg.as_ref() as *const Mutex<Instruction>;
                    let arg_id = if let Some(id) = map.get(&p) {
                        *id
                    } else {
                        88888
                    };
                    arg_ids.push(arg_id);
                }
                let arg_strings: Vec<String> = arg_ids.iter().map(|i| format!("%{}", *i)).collect();
                let args_connected = arg_strings.join(", ");

                self.print(format!("%{} = %{}({})", id, function_id, args_connected))
            }
            Instruction::Return(ref ret) => {
                let value = ret.instruction.as_ref() as *const Mutex<Instruction>;
                let value_id = if let Some(id) = map.get(&value) {
                    *id
                } else {
                    77777
                };

                self.print(format!("ret %{}", value_id))
            }
            Instruction::GetParameter(ref param) => {
                self.print(format!("%{} = param %{}", id, param.name))
            }
            _ => self.print(format!("%{} = unprintable", id)),
        }
    }
}
