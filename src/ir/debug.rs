use core::borrow::{Borrow, BorrowMut};
use std::collections::HashMap;
use std::env::var;
use std::sync::{Arc, Mutex, MutexGuard};

use crate::ir::*;

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
            Declaration::Actor(ref actor) => {
                self.print_actor(actor);
            }
            Declaration::Struct(ref struc) => {
                self.print_struct(struc);
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

    pub fn print_struct(&mut self, struc: &Box<Struct>) {
        self.print(format!("struct {}:", struc.name));
        self.push();

        let traits = struc.traits.read().unwrap();
        for trai in traits.iter() {
            self.print(format!("+trait {}", trai.name()))
        }

        let fields = struc.fields.read().unwrap();
        for field in fields.iter() {
            let field_guard = field.0.lock().unwrap();
            if let Some(ref field) = *field_guard {
                self.print_declaration(field);
            }
        }

        let functions = struc.functions.read().unwrap();
        for function in functions.iter() {
            let function_guard = function.0.lock().unwrap();
            if let Some(ref function) = *function_guard {
                self.print_declaration(function);
            }
        }
        self.pop();
    }

    pub fn print_actor(&mut self, actor: &Box<Actor>) {
        self.print(format!("actor {}:", actor.name));
        self.push();

        let fields = actor.fields.read().unwrap();
        for field in fields.iter() {
            let field_guard = field.0.lock().unwrap();
            if let Some(ref field) = *field_guard {
                self.print_declaration(field);
            }
        }

        let functions = actor.functions.read().unwrap();
        for function in functions.iter() {
            let function_guard = function.0.lock().unwrap();
            if let Some(ref function) = *function_guard {
                self.print_declaration(function);
            }
        }

        let behaviours = actor.behaviours.read().unwrap();
        for behaviour in behaviours.iter() {
            let behaviour_guard = behaviour.0.lock().unwrap();
            if let Some(ref behaviour) = *behaviour_guard {
                self.print_declaration(behaviour);
            }
        }
        self.pop();
    }

    pub fn print_variable(&mut self, variable: &Box<Variable>) {
        self.print(format!("let {}", variable.name));
    }

    pub fn print_function(&mut self, function: &Box<Function>) {
        let mut joined_args = "".to_string();
        for (id, (arg, dec)) in (&function.arguments).iter().enumerate() {
            joined_args.push_str(&format!("%{}: {}", arg, dec.name()));
            if id < function.arguments.len() - 1 {
                joined_args.push_str(", ");
            }
        }

        let return_type_name = &function.return_type.name();
        self.print(format!(
            "fun @{}({}) -> {}:",
            function.name, joined_args, return_type_name
        ));
        self.push();

        if function.blocks.len() > 0 {
            self.print(format!("body:"));
            self.push();
            for (id, block) in function.blocks.iter().enumerate() {
                self.print_block(id, block);
            }
            self.pop();
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
        let ins_type = instruction.get_type().name();
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

                self.print(format!("%{} : {} = @{}::{}{}", id, ins_type, path_name, name, filled))
            }
            Instruction::IntegerLiteral(ref i) => self.print(format!("%{} : {} = {}", id, ins_type, i.as_ref().0)),
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

                self.print(format!("%{} : {} = %{}({})", id, ins_type, function_id, args_connected))
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
                self.print(format!("%{} : {} = param %{}", id, ins_type, param.name))
            }
            _ => self.print(format!("%{} : {} = unprintable", id, ins_type)),
        }
    }
}
