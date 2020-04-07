use std::collections::HashMap;

use crate::ir::*;
use crate::util::Either;

pub enum PrintMode {
    Stdout,
    Buffer,
}

pub struct Printer {
    depth: usize,
    mode: PrintMode,
    pub buffer: String,
}

impl Printer {
    pub fn new(mode: PrintMode) -> Self {
        Self {
            depth: 0,
            mode,
            buffer: String::new(),
        }
    }

    pub fn push(&mut self) {
        self.depth += 1;
    }

    pub fn pop(&mut self) {
        self.depth -= 1;
    }

    pub fn print(&mut self, s: String) {
        match self.mode {
            PrintMode::Stdout => {
                if self.depth > 0 {
                    print!("{}", "  ".repeat(self.depth));
                }
                println!("{}", s);
            }
            PrintMode::Buffer => {
                if self.depth > 0 {
                    self.buffer.push_str(&"  ".repeat(self.depth));
                }
                self.buffer.push_str(&format!("{}\n", s));
            }
        }
    }

    pub fn print_module<'compiler>(&mut self, compiler: &'compiler Compiler<'compiler>, module: &Module<'compiler>) {
        println!("test");
        self.print(format!(
            "; Module: {}::{}",
            module.path.to_string(),
            module.name
        ));
        for declaration in module.declarations.iter() {
            self.print_declaration(compiler, declaration);
        }
    }

    pub fn print_declaration<'compiler>(&mut self, compiler: &'compiler Compiler<'compiler>, dec: &Declaration<'compiler>) {
        match dec {
            Declaration::Actor(ref actor) => {
                self.print_actor(compiler, actor);
            }
            Declaration::Struct(ref struc) => {
                self.print_struct(compiler, struc);
            }
            Declaration::Behaviour(ref behaviour) => {
                self.print_behaviour(compiler, behaviour);
            }
            Declaration::Function(ref function) => {
                self.print_function(compiler, function);
            }
            Declaration::Variable(ref variable) => {
                self.print_variable(variable);
            }
            _ => {}
        }
    }

    pub fn print_struct<'compiler>(&mut self, compiler: &'compiler Compiler<'compiler>, struc: &Box<Struct<'compiler>>) {
        self.print(format!("struct {}:", struc.name));
        self.push();

        for trai in struc.traits.iter() {
            self.print(format!("+trait {}", trai.name()))
        }

        for field in struc.fields.iter() {
            self.print_declaration(compiler, field);
        }

        for function in struc.functions.iter() {
            self.print_declaration(compiler, function);
        }
        self.pop();
    }

    pub fn print_actor<'compiler>(&mut self, compiler: &'compiler Compiler<'compiler>, actor: &Box<Actor<'compiler>>) {
        self.print(format!("actor {}:", actor.name));
        self.push();

        for field in actor.fields.iter() {
            self.print_declaration(compiler, field);
        }

        for function in actor.functions.iter() {
            self.print_declaration(compiler, function);
        }

        for behaviour in actor.behaviours.iter() {
            self.print_declaration(compiler, behaviour);
        }
        self.pop();
    }

    pub fn print_variable(&mut self, variable: &Box<Variable>) {
        self.print(format!("let {}: {}", variable.name, variable.typ.get_type().name().clone()));
    }

    pub fn print_behaviour<'compiler>(&mut self, compiler: &'compiler Compiler<'compiler>, behaviour: &Box<Behaviour<'compiler>>) {
        let mut joined_args = "".to_string();
        for (id, (arg, dec)) in (&behaviour.arguments).iter().enumerate() {
            joined_args.push_str(&format!("%{}: {}", arg, dec.name()));
            if id < behaviour.arguments.len() - 1 {
                joined_args.push_str(", ");
            }
        }

        self.print(format!(
            "behave @{}({}):",
            behaviour.name, joined_args
        ));
        self.push();

        if behaviour.blocks.len() > 0 {
            let mut instruction_ids: HashMap<*const Instruction, usize> = HashMap::new();
            let mut block_ids: HashMap<*const Block, usize> = HashMap::new();
            // put block ids in map
            for (id, block) in behaviour.blocks.iter().enumerate() {
                block_ids.insert(block as *const Block, id);
            }
            for (id, block) in behaviour.blocks.iter().enumerate() {
                self.print_block(compiler, &mut instruction_ids, &block_ids, id, block, Either::Right(behaviour));
            }
        }
        self.pop();
    }

    pub fn print_function<'compiler>(&mut self, compiler: &'compiler Compiler<'compiler>, function: &Box<Function<'compiler>>) {
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
            let mut instruction_ids: HashMap<*const Instruction, usize> = HashMap::new();
            let mut block_ids: HashMap<*const Block, usize> = HashMap::new();
            // put block ids in map
            for (id, block) in function.blocks.iter().enumerate() {
                block_ids.insert(block as *const Block, id);
            }

            for (id, block) in function.blocks.iter().enumerate() {
                self.print_block(compiler, &mut instruction_ids, &block_ids, id, block, Either::Left(function));
            }
        }
        self.pop();
    }

    pub fn print_block<'compiler>(
        &mut self,
        compiler: &'compiler Compiler<'compiler>,
        instruction_ids: &mut HashMap<*const Instruction<'compiler>, usize>,
        block_ids: &HashMap<*const Block, usize>,
        id: usize,
        block: &Block<'compiler>,
        function: Either<&Box<Function>, &Box<Behaviour>>,
    ) {
        self.print(format!("block#{}:", id));

        self.push();
        for instruction in block.instructions.iter() {
            let id = instruction_ids.len();
            instruction_ids.insert(instruction as *const Instruction, id);
            self.print_instruction(compiler, &instruction_ids, block_ids, id, instruction, function);
        }
        self.pop();
    }

    pub fn print_instruction<'compiler>(
        &mut self,
        compiler: &'compiler Compiler<'compiler>,
        instruction_ids: &HashMap<*const Instruction<'compiler>, usize>,
        block_ids: &HashMap<*const Block, usize>,
        id: usize,
        instruction: &Instruction<'compiler>,
        function: Either<&Box<Function>, &Box<Behaviour>>,
    ) {
        let ins_type = instruction.get_type_with_context(compiler, function).name();
        match *instruction {
            Instruction::DeclarationReference(ref d) => {
                let (path, name) = &d.name;
                let path_name = match path {
                    Some(path) => path.to_string(),
                    None => "".to_string(),
                };
                let filled = match compiler.resolve(&d.declaration) {
                    None => "*",
                    Some(_) => "",
                };

                self.print(format!("%{} : {} = @{}::{}{}", id, ins_type, path_name, name, filled))
            }
            Instruction::BooleanLiteral(ref b) => self.print(format!("%{} : {} = {}", id, ins_type, b.as_ref().0)),
            Instruction::IntegerLiteral(ref i) => self.print(format!("%{} : {} = {}", id, ins_type, i.as_ref().0)),
            Instruction::FunctionCall(ref call) => {
                let function = call.function as *const Instruction;
                let function_id = if let Some(id) = instruction_ids.get(&function) {
                    *id
                } else {
                    99999
                };

                let mut arg_ids = Vec::with_capacity(call.arguments.len());
                for arg in call.arguments.iter() {
                    let p = (*arg) as *const Instruction;
                    let arg_id = if let Some(id) = instruction_ids.get(&p) { *id } else { 88888 };
                    arg_ids.push(arg_id);
                }
                let arg_strings: Vec<String> = arg_ids.iter().map(|i| format!("%{}", *i)).collect();
                let args_connected = arg_strings.join(", ");

                self.print(format!("%{} : {} = %{}({})", id, ins_type, function_id, args_connected))
            }
            Instruction::Return(ref ret) => {
                let value = ret.instruction as *const Instruction;
                let value_id = if let Some(id) = instruction_ids.get(&value) { *id } else { 77777 };

                self.print(format!("ret %{}", value_id))
            }
            Instruction::GetParameter(ref param) => {
                self.print(format!("%{} : {} = param %{}", id, ins_type, param.name))
            }
            Instruction::Branch(ref branch) => {
                let cond = branch.condition as *const Instruction;
                let cond_id = if let Some(id) = instruction_ids.get(&cond) { *id } else { 66666 };

                let true_block = branch.true_block as *const Block;
                let false_block = branch.false_block as *const Block;

                let true_block_id = if let Some(id) = block_ids.get(&true_block) { *id } else { 66665 };
                let false_block_id = if let Some(id) = block_ids.get(&false_block) { *id } else { 66664 };

                self.print(format!("branch %{} block#{} block#{}", cond_id, true_block_id, false_block_id))
            }
            Instruction::Jump(ref jump) => {
                let block = jump.block as *const Block;

                let block_id = if let Some(id) = block_ids.get(&block) { *id } else { 66663 };
                self.print(format!("jump block#{}", block_id))
            }
            ref i => self.print(format!("%{} : {} = unprintable ({:?})", id, ins_type, i)),
        }
    }
}
