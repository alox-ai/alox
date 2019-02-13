use crate::ast;
use crate::ir;
use crate::ir::types::Type;

pub fn generate_ir(program: ast::Program) -> ir::Module {
    // args for module
    let name = program.file_name;
    let mut function_headers = vec![];
    let mut functions = vec![];

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
                function_headers.push(ir::FunctionHeader {
                    name,
                    arguments,
                    return_type,
                    refinements,
                    permissions,
                });
            }
            ast::Node::FunctionDefinition(f) => {}
            ast::Node::VariableDeclaration(v) => {}
        }
    }

    ir::Module {
        name,
        function_headers,
        functions,
    }
}