use std::sync::{Arc, Mutex};

use crate::ast;
use crate::ir;
use crate::ir::types::Type;

impl ir::Compiler {
    pub fn generate_ir(&self, program: ast::Program) -> ir::Module {
        // args for module
        let name = program.file_name;
        let mut declarations: Vec<Arc<ir::Declaration>> = vec![];

        // go over each node and generate the ir
        for mut node in program.nodes {
            match node {
                // todo
                ast::Node::Struct(s) => {}
                ast::Node::Trait(t) => {}

                ast::Node::FunctionDeclaration(mut f) => {
                    let name = f.name;
                    let arguments: Vec<(String, Type)> = f.arguments.iter_mut()
                        .map(|(name, t)| {
                            (name.clone(), Type::Unresolved(t.clone()))
                        }).collect();
                    let return_type = Type::Unresolved(f.return_type);
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
                        header: Arc::new(Mutex::new(header)),
                        blocks,
                    }));
                    declarations.push(Arc::new(function));
                }
                ast::Node::VariableDeclaration(v) => {}
            }
        }

        ir::Module {
            name,
            declarations,
        }
    }
}
