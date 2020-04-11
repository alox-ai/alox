use crate::ir::*;

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

    pub fn print_module(&mut self, compiler: &Compiler, module: &Module) {
        self.print(format!(
            "; Module: {}::{}",
            module.path.to_string(),
            module.name
        ));
        for declaration in module.declarations.iter() {
            self.print_declaration(compiler, declaration);
        }
    }

    pub fn print_declaration(&mut self, compiler: &Compiler, dec: &Declaration) {
        match dec {
            Declaration::Struct(ref struc) => {
                self.print_struct(compiler, struc);
            }
            Declaration::Function(ref function) => {
                self.print_function(compiler, function);
            }
            Declaration::Variable(ref variable) => {
                self.print_variable(compiler, variable);
            }
            _ => {}
        }
    }

    pub fn print_struct(&mut self, compiler: &Compiler, struc: &Box<Struct>) {
        self.print(format!("{} {}:", struc.kind.name(), struc.name));
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

    pub fn print_variable(&mut self, compiler: &Compiler, variable: &Variable) {
        self.print(format!("let {}: {}", variable.name, variable.typ.get_type(compiler).name().clone()));
    }

    pub fn print_function(&mut self, compiler: &Compiler, function: &Function) {
        let mut joined_args = "".to_string();
        for (id, (arg, dec)) in (&function.arguments).iter().enumerate() {
            joined_args.push_str(&format!("%{}: {}", arg, dec.name()));
            if id < function.arguments.len() - 1 {
                joined_args.push_str(", ");
            }
        }

        let return_type_name = &function.return_type.name();
        self.print(format!(
            "{} @{}({}) -> {}:",
            function.kind.name(), function.name, joined_args, return_type_name
        ));
        self.push();

        if function.blocks.len() > 0 {
            for (id, block) in function.blocks.iter().enumerate() {
                self.print_block(compiler, id, block, function);
            }
        }
        self.pop();
    }

    pub fn print_block(
        &mut self,
        compiler: &Compiler,
        id: usize,
        block: &Block,
        function: &Function,
    ) {
        self.print(format!("block#{}:", id));

        self.push();
        for (id, instruction) in block.instructions.iter().enumerate() {
            self.print_instruction(compiler, id, instruction, block, function);
        }
        self.pop();
    }

    pub fn print_instruction(
        &mut self,
        compiler: &Compiler,
        id: usize,
        instruction: &Instruction,
        block: &Block,
        function: &Function,
    ) {
        let ins_type = instruction.get_type_with_context(compiler, block, function).name();
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
                let function_id = call.function;
                let arg_strings: Vec<String> = call.arguments.iter().map(|i| format!("%{}", (*i).0)).collect();
                let args_connected = arg_strings.join(", ");

                self.print(format!("%{} : {} = %{}({})", id, ins_type, function_id.0, args_connected))
            }
            Instruction::Return(ref ret) => {
                let value_id = ret.instruction;

                self.print(format!("ret %{}", value_id.0))
            }
            Instruction::GetParameter(ref param) => {
                self.print(format!("%{} : {} = param %{}", id, ins_type, param.name))
            }
            Instruction::Branch(ref branch) => {
                let cond_id = branch.condition;

                let true_block_id = branch.true_block;
                let false_block_id = branch.false_block;

                self.print(format!("branch %{} block#{} block#{}", cond_id.0, true_block_id.0, false_block_id.0))
            }
            Instruction::Jump(ref jump) => {
                let block_id = jump.block;
                self.print(format!("jump block#{}", block_id.0))
            }
            Instruction::Load(ref load) => {
                self.print(format!("%{} : {} = load %{}", id, ins_type, load.name))
            }
            Instruction::Store(ref store) => {
                self.print(format!("store %{} in %{}", store.value.0, store.name))
            }
            Instruction::Alloca(ref alloca) => {
                let inner_type = block.get_instruction(alloca.reference_ins).get_type(compiler, block);
                self.print(format!("%{} : {} = alloca {}", alloca.name, ins_type, inner_type.name()))
            }
            ref i => self.print(format!("%{} : {} = unprintable ({:?})", id, ins_type, i)),
        }
    }
}
