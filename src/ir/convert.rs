use std::sync::{Arc, Mutex};

use crate::ast;
use crate::ir;
use crate::ir::types::Type;

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
                ast::Node::Struct(s) => {}
                ast::Node::Trait(t) => {}

                ast::Node::FunctionDeclaration(mut f) => {
                    let name = f.name;
                    let mut arguments: Vec<(String, ir::DeclarationWrapper)> = Vec::with_capacity(f.arguments.len());
                    for (name, type_path) in f.arguments {
                        let typ = self.resolve(type_path.0, type_path.1, ir::DeclarationKind::Type);
                        arguments.push((name, typ));
                    }
                    let return_type = self.resolve(f.return_type.0, f.return_type.1, ir::DeclarationKind::Type);
                    let refinements = vec![];
                    let permissions = vec![];
                    let function_header = ir::Declaration::FunctionHeader(Box::new(ir::FunctionHeader {
                        name,
                        arguments,
                        return_type,
                        refinements,
                        permissions,
                    }));
                    declarations.push(Arc::new(function_header));
                }
                ast::Node::FunctionDefinition(f) => {
                    let name = f.name;
                    let blocks: Vec<Arc<Mutex<ir::Block>>> = vec![];

                    let mut header = None;
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
                    let function = ir::Declaration::Function(Box::new(ir::Function {
                        name,
                        header: Arc::new(Mutex::new(header)),
                        blocks,
                    }));
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
}
