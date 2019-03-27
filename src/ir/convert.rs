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
        let mut blocks: Vec<ir::Block> = vec![];
        let mut current_block: ir::Block = ir::Block::new();

        // TODO: instructions
        for statement in f.statements.iter() {
            match statement {
                ast::Statement::Return(ret) => {
                    // ir::Instruction::Return(Box::new(ir::))
                    blocks.push(current_block);
                    current_block = ir::Block::new();
                }
                _ => {}
            }
        }

        let mut blocks_wrapped = Vec::with_capacity(blocks.len());
        for b in blocks {
            blocks_wrapped.push(Arc::new(Mutex::new(b)));
        }

        ir::Declaration::Function(Box::new(ir::Function {
            name,
            header: Arc::new(Mutex::new(header)),
            blocks: blocks_wrapped,
        }))
    }
}
