use std::collections::HashMap;
use std::rc::Rc;
use std::sync::{Arc, Mutex, RwLock};

use crate::ast;
use crate::ast::FunctionDeclaration;
use crate::ir;

impl ir::Compiler {
    pub fn generate_ir(&self, program: ast::Program) -> ir::Module {
        // args for module
        let name = program.file_name;
        let path = program.path;
        let mut declarations: Vec<Arc<ir::Declaration>> = vec![];

        // go over each node and generate the ir
        for mut node in program.nodes {
            match node {
                // todo
                ast::Node::Struct(s) => {
                    let mut fields = Vec::with_capacity(s.fields.len());
                    let mut traits = Vec::with_capacity(s.traits.len());
                    let mut function_headers_unwrapped = Vec::with_capacity(s.function_declarations.len());
                    let mut functions = Vec::with_capacity(s.function_definitions.len());

                    for f in s.function_declarations {
                        let function_header = self.generate_ir_function_header(&f);
                        function_headers_unwrapped.push(Arc::new(function_header));
                    }

                    for f in s.function_definitions {
                        let mut header: Option<Arc<ir::Declaration>> = None;
                        for declaration in function_headers_unwrapped.iter() {
                            match *declaration.clone() {
                                ir::Declaration::FunctionHeader(ref f) => {
                                    if f.name == name {
                                        header = Some(declaration.clone());
                                    }
                                }
                                _ => {}
                            }
                        }
                        if let None = header {
                            for declaration in declarations.iter() {
                                match *declaration.clone() {
                                    ir::Declaration::FunctionHeader(ref f) => {
                                        if f.name == name {
                                            header = Some(declaration.clone());
                                        }
                                    }
                                    _ => {}
                                }
                            }
                        }
                        let function = self.generate_ir_function_definition(&declarations, &f, header);
                        functions.push(ir::wrap_declaration(function));
                    }

                    // wrap headers
                    let function_headers: Vec<ir::DeclarationWrapper> = function_headers_unwrapped.iter_mut()
                        .map(|f| Arc::new(Mutex::new(Some((*f).clone()))))
                        .collect();

                    let strct = ir::Struct {
                        name: s.name,
                        fields: Arc::new(RwLock::new(fields)),
                        traits: Arc::new(RwLock::new(traits)),
                        function_headers: Arc::new(RwLock::new(function_headers)),
                        functions: Arc::new(RwLock::new(functions)),
                    };
                    declarations.push(Arc::new(ir::Declaration::Struct(Box::new(strct))));
                }
                ast::Node::Trait(t) => {}

                ast::Node::FunctionDeclaration(mut f) => {
                    let function_header = self.generate_ir_function_header(f.as_ref());
                    declarations.push(Arc::new(function_header));
                }
                ast::Node::FunctionDefinition(f) => {
                    let mut header: Option<Arc<ir::Declaration>> = None;
                    for declaration in declarations.iter() {
                        match *declaration.clone() {
                            ir::Declaration::FunctionHeader(ref f) => {
                                if f.name == name {
                                    header = Some(declaration.clone());
                                }
                            }
                            _ => {}
                        }
                    }

                    let function = self.generate_ir_function_definition(&declarations, f.as_ref(), header);
                    declarations.push(Arc::new(function));
                }
                ast::Node::VariableDeclaration(v) => {}
            }
        }

        ir::Module {
            path,
            name,
            declarations,
        }
    }

    pub fn generate_ir_function_header(&self, f: &ast::FunctionDeclaration) -> ir::Declaration {
        let name = f.name.clone();
        let mut arguments: Vec<(String, ir::DeclarationWrapper)> = Vec::with_capacity(f.arguments.len());
        for (name, type_path) in f.arguments.iter() {
            let typ = self.resolve(type_path.0.clone(), type_path.1.clone(), ir::DeclarationKind::Type);
            arguments.push((name.clone(), typ));
        }
        let return_type = self.resolve(f.return_type.0.clone(), f.return_type.1.clone(), ir::DeclarationKind::Type);
        let refinements = vec![];
        let permissions = vec![];
        ir::Declaration::FunctionHeader(Box::new(ir::FunctionHeader {
            name,
            arguments,
            return_type,
            refinements,
            permissions,
        }))
    }

    pub fn generate_ir_function_definition(&self, declarations: &Vec<Arc<ir::Declaration>>, f: &ast::FunctionDefinition, header: Option<Arc<ir::Declaration>>) -> ir::Declaration {
        let name = f.name.clone();
        let mut block_builder = BlockBuilder::new();
        let mut lvt = LocalVariableTable::new();

        for statement in f.statements.iter() {
            self.generate_ir_statement(&mut lvt, &mut block_builder, statement);
        }

        let blocks = block_builder.blocks;
        let mut blocks_wrapped = Vec::with_capacity(blocks.len());
        for b in blocks {
            // this might come back to bite me
            if b.instructions.len() > 0 {
                blocks_wrapped.push(Arc::new(Mutex::new(b)));
            }
        }

        ir::Declaration::Function(Box::new(ir::Function {
            name,
            header: Arc::new(Mutex::new(header)),
            blocks: blocks_wrapped,
        }))
    }

    pub fn generate_ir_statement(&self, lvt: &mut LocalVariableTable, block_builder: &mut BlockBuilder, statement: &ast::Statement) {
        match statement {
            ast::Statement::VariableDeclaration(d) => {
                let expr_ins = self.generate_ir_expression(lvt, block_builder.current_block(), &d.initial_expression);
                lvt.set(d.name.clone(), expr_ins);
            }
            ast::Statement::Return(r) => {
                let expr_ins = self.generate_ir_expression(lvt, block_builder.current_block(), &r.expression);
                let return_ins = Arc::new(Mutex::new(ir::Instruction::Return(Box::new(ir::Return { instruction: expr_ins }))));
                block_builder.add_instruction(return_ins);
                block_builder.create_block();
            }
            _ => {}
        }
    }

    pub fn generate_ir_expression(&self, lvt: &mut LocalVariableTable, block: &mut ir::Block, expression: &ast::Expression) -> Arc<Mutex<ir::Instruction>> {
        let ins = match expression {
            ast::Expression::IntegerLiteral(i) => {
                let ins = Arc::new(Mutex::new(ir::Instruction::IntegerLiteral(Box::new(ir::IntegerLiteral(i.as_ref().0)))));
                ins
            }
            ast::Expression::FunctionCall(call) => {
                let function = self.generate_ir_expression(lvt, block, &call.function);
                let mut arguments = Vec::with_capacity(call.arguments.len());
                for argument in call.arguments.iter() {
                    let argument_ins = self.generate_ir_expression(lvt, block, argument);
                    arguments.push(argument_ins);
                }
                Arc::new(Mutex::new(ir::Instruction::FunctionCall(Box::new(ir::FunctionCall {
                    function,
                    arguments,
                }))))
            }
            ast::Expression::VariableReference(r) => {
                let name = r.name.clone();
                if let Some(path) = &r.path {
                    // this is a declaration to something in a module
                    Arc::new(Mutex::new(ir::Instruction::DeclarationReference(Box::new(ir::DeclarationReference::blank_with_path(path.clone(), name)))))
                } else {
                    // this is a local variable
                    if let Some(ins) = lvt.get(name.clone()) {
                        ins
                    } else {
                        Arc::new(Mutex::new(ir::Instruction::DeclarationReference(Box::new(ir::DeclarationReference::blank(name)))))
//                        let debug = format!("VariableReference({})", name);
//                        Arc::new(Mutex::new(ir::Instruction::Unreachable(debug)))
                    }
                }
            }
            e => Arc::new(Mutex::new(ir::Instruction::Unreachable(format!("UnhandledExpression({})", e.name()))))
        };
        block.add_instruction(ins.clone());
        ins
    }
}

struct BlockBuilder {
    blocks: Vec<ir::Block>,
    current_block: usize,
}

impl BlockBuilder {
    pub fn new() -> Self {
        let mut blocks = Vec::new();
        blocks.push(ir::Block::new());
        Self {
            blocks,
            current_block: 0,
        }
    }

    pub fn current_block(&mut self) -> &mut ir::Block {
        self.blocks.get_mut(self.current_block).unwrap()
    }

    pub fn create_block(&mut self) -> &ir::Block {
        self.blocks.push(ir::Block::new());
        self.current_block()
    }

    pub fn add_instruction(&mut self, instruction: Arc<Mutex<ir::Instruction>>) {
        self.blocks.get_mut(self.current_block).unwrap().instructions.push(instruction);
    }
}

#[derive(Debug)]
struct LocalVariableTable {
    table: Vec<HashMap<String, Arc<Mutex<ir::Instruction>>>>
}

impl LocalVariableTable {
    pub fn new() -> Self {
        let mut table = Vec::new();
        table.push(HashMap::new());
        Self {
            table
        }
    }

    pub fn push_depth(&mut self) {
        self.table.push(HashMap::new());
    }

    pub fn pop_depth(&mut self) {
        self.table.pop();
    }

    pub fn get(&self, name: String) -> Option<Arc<Mutex<ir::Instruction>>> {
        for x in self.table.iter().rev() {
            if let Some(instruction) = x.get(&name) {
                return Some(instruction.clone());
            }
        }
        None
    }

    pub fn set(&mut self, name: String, instruction: Arc<Mutex<ir::Instruction>>) {
        if let Some(mut map) = self.table.last_mut() {
            map.insert(name, instruction);
        }
    }
}
