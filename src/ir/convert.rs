use std::collections::HashMap;
use std::rc::Rc;
use std::sync::{Arc, Mutex, RwLock};
use std::time::Instant;

use crate::ast;
use crate::ir;
use crate::ir::{DeclarationContainer, DeclarationKind};

impl ir::Compiler {
    pub fn generate_ir(&self, program: ast::Program) -> ir::Module {
        // args for module
        let name = program.file_name;
        let path = program.path;
        let current_path = &path.append(name.clone());
        let mut completed_declarations: Vec<Arc<ir::Declaration>> = vec![];

        // go over each node and generate the ir
        for mut node in program.nodes {
            match node {
                ast::Node::Actor(a) => {
                    let mut fields = Vec::with_capacity(a.fields.len());
                    let mut functions = Vec::with_capacity(a.functions.len());
                    let mut behaviours = Vec::with_capacity(a.behaviours.len());

                    for field in a.fields {
                        let declaration = if let Some(type_path) = field.type_name {
                            self.resolve(
                                type_path.0.clone(),
                                type_path.1.clone(),
                                Some(ir::DeclarationKind::Type),
                            )
                        } else {
                            // TODO better errors
                            panic!("No type on field")
                        };
                        let variable = ir::DeclarationContainer::from(ir::Declaration::Variable(Box::new(ir::Variable {
                            name: field.name,
                            typ: declaration,
                        })));
                        fields.push(variable);
                    }

                    for f in a.functions {
                        let function = self.generate_ir_function(
                            current_path,
                            &completed_declarations,
                            &f,
                        );
                        functions.push(ir::DeclarationContainer::from(function));
                    }

                    for b in a.behaviours {
                        let behaviour = self.generate_ir_behaviour(
                            current_path,
                            &completed_declarations,
                            &b,
                        );
                        behaviours.push(ir::DeclarationContainer::from(behaviour));
                    }

                    let actor = ir::Actor {
                        name: a.name,
                        fields: Arc::new(RwLock::new(fields)),
                        behaviours: Arc::new(RwLock::new(behaviours)),
                        functions: Arc::new(RwLock::new(functions)),
                    };
                    completed_declarations.push(Arc::new(ir::Declaration::Actor(Box::new(actor))));
                }
                ast::Node::Struct(s) => {
                    let mut fields = Vec::with_capacity(s.fields.len());
                    let mut traits = Vec::with_capacity(s.traits.len());
                    let mut functions = Vec::with_capacity(s.functions.len());

                    for field in s.fields {
                        let declaration = if let Some(type_path) = field.type_name {
                            self.resolve(
                                type_path.0.clone(),
                                type_path.1.clone(),
                                Some(ir::DeclarationKind::Type),
                            )
                        } else {
                            // TODO better errors
                            panic!("No type on field")
                        };
                        let variable = ir::DeclarationContainer::from(ir::Declaration::Variable(Box::new(ir::Variable {
                            name: field.name,
                            typ: declaration,
                        })));
                        fields.push(variable);
                    }

                    for f in s.functions {
                        let function = self.generate_ir_function(
                            current_path,
                            &completed_declarations,
                            &f,
                        );
                        functions.push(ir::DeclarationContainer::from(function));
                    }

                    let strct = ir::Struct {
                        name: s.name,
                        fields: Arc::new(RwLock::new(fields)),
                        traits: Arc::new(RwLock::new(traits)),
                        functions: Arc::new(RwLock::new(functions)),
                    };
                    completed_declarations.push(Arc::new(ir::Declaration::Struct(Box::new(strct))));
                }
                ast::Node::Trait(t) => {}
                ast::Node::Function(f) => {
                    let function = self.generate_ir_function(
                        current_path,
                        &completed_declarations,
                        f.as_ref(),
                    );
                    completed_declarations.push(Arc::new(function));
                }
                ast::Node::VariableDeclaration(v) => {}
            }
        }

        ir::Module {
            path,
            name,
            declarations: completed_declarations,
        }
    }

    pub fn generate_ir_behaviour(
        &self,
        current_path: &ast::Path,
        declarations: &Vec<Arc<ir::Declaration>>,
        b: &ast::Behaviour,
    ) -> ir::Declaration {
        let name = b.name.clone();

        // get the parameters from the function header
        let mut param_names = vec![];
        for (name, dec) in &b.arguments {
            param_names.push(name.clone());
        }

        let mut block_builder = BlockBuilder::new();
        let mut lvt = LocalVariableTable::new_with_params(param_names);

        for statement in b.statements.iter() {
            self.generate_ir_statement(
                current_path,
                declarations,
                &mut lvt,
                &mut block_builder,
                statement,
            );
        }

        let blocks = block_builder.blocks;
        let mut blocks_wrapped = Vec::with_capacity(blocks.len());
        for b in blocks {
            // this might come back to bite me
            if b.instructions.len() > 0 {
                blocks_wrapped.push(Arc::new(Mutex::new(b)));
            }
        }
        let mut arguments = Vec::with_capacity(b.arguments.len());
        for (name, type_path) in b.arguments.iter() {
            let typ = self.resolve(
                type_path.0.clone(),
                type_path.1.clone(),
                Some(ir::DeclarationKind::Type),
            );
            arguments.push((name.clone(), typ));
        }

        ir::Declaration::Behaviour(Box::new(ir::Behaviour {
            name,
            arguments,
            blocks: blocks_wrapped,
        }))
    }

    pub fn generate_ir_function(
        &self,
        current_path: &ast::Path,
        declarations: &Vec<Arc<ir::Declaration>>,
        f: &ast::Function,
    ) -> ir::Declaration {
        let name = f.name.clone();

        // get the parameters from the function header
        let mut param_names = vec![];
        for (name, dec) in &f.arguments {
            param_names.push(name.clone());
        }

        let mut block_builder = BlockBuilder::new();
        let mut lvt = LocalVariableTable::new_with_params(param_names);

        for statement in f.statements.iter() {
            self.generate_ir_statement(
                current_path,
                declarations,
                &mut lvt,
                &mut block_builder,
                statement,
            );
        }

        let blocks = block_builder.blocks;
        let mut blocks_wrapped = Vec::with_capacity(blocks.len());
        for b in blocks {
            // this might come back to bite me
            if b.instructions.len() > 0 {
                blocks_wrapped.push(Arc::new(Mutex::new(b)));
            }
        }
        let mut arguments = Vec::with_capacity(f.arguments.len());
        for (name, type_path) in f.arguments.iter() {
            let typ = self.resolve(
                type_path.0.clone(),
                type_path.1.clone(),
                Some(ir::DeclarationKind::Type),
            );
            arguments.push((name.clone(), typ));
        }

        let return_type = self.resolve(
            f.return_type.0.clone(),
            f.return_type.1.clone(),
            Some(ir::DeclarationKind::Type),
        );

        ir::Declaration::Function(Box::new(ir::Function {
            name,
            arguments,
            return_type,
            blocks: blocks_wrapped,
        }))
    }

    pub fn generate_ir_statement(
        &self,
        current_path: &ast::Path,
        completed_declarations: &Vec<Arc<ir::Declaration>>,
        lvt: &mut LocalVariableTable,
        block_builder: &mut BlockBuilder,
        statement: &ast::Statement,
    ) {
        match statement {
            ast::Statement::VariableDeclaration(d) => {
                if let Some(exp) = &d.initial_expression {
                    let expr_ins = self.generate_ir_expression(
                        current_path,
                        completed_declarations,
                        lvt,
                        block_builder.current_block(),
                        exp,
                        None,
                    );
                    lvt.set(d.name.clone(), expr_ins);
                } else {
                    lvt.set(d.name.clone(), Arc::new(Mutex::new(ir::Instruction::Unreachable("MissingInitialExpression".to_string()))));
                }
            }
            ast::Statement::Return(r) => {
                let expr_ins = self.generate_ir_expression(
                    current_path,
                    completed_declarations,
                    lvt,
                    block_builder.current_block(),
                    &r.expression,
                    None,
                );
                let return_ins =
                    Arc::new(Mutex::new(ir::Instruction::Return(Box::new(ir::Return {
                        instruction: expr_ins,
                    }))));
                block_builder.add_instruction(return_ins);
                block_builder.create_block();
            }
            ast::Statement::FunctionCall(call) => {
                let function = self.generate_ir_expression(
                    current_path,
                    completed_declarations,
                    lvt,
                    block_builder.current_block(),
                    &call.function,
                    Some(ir::DeclarationKind::Function),
                );
                let mut arguments = Vec::with_capacity(call.arguments.len());
                for argument in call.arguments.iter() {
                    let argument_ins = self.generate_ir_expression(
                        current_path,
                        completed_declarations,
                        lvt,
                        block_builder.current_block(),
                        argument,
                        None,
                    );
                    arguments.push(argument_ins);
                }
                let call_ins = Arc::new(Mutex::new(ir::Instruction::FunctionCall(Box::new(
                    ir::FunctionCall {
                        function,
                        arguments,
                    },
                ))));
                block_builder.add_instruction(call_ins);
            }
            _ => {}
        }
    }

    pub fn generate_ir_expression(
        &self,
        current_path: &ast::Path,
        completed_declarations: &Vec<Arc<ir::Declaration>>,
        lvt: &mut LocalVariableTable,
        block: &mut ir::Block,
        expression: &ast::Expression,
        declaration_context: Option<ir::DeclarationKind>,
    ) -> Arc<Mutex<ir::Instruction>> {
        let ins = match expression {
            ast::Expression::IntegerLiteral(i) => Arc::new(Mutex::new(
                ir::Instruction::IntegerLiteral(Box::new(ir::IntegerLiteral(i.as_ref().0))),
            )),
            ast::Expression::FunctionCall(call) => {
                let function = self.generate_ir_expression(
                    current_path,
                    completed_declarations,
                    lvt,
                    block,
                    &call.function,
                    Some(ir::DeclarationKind::Function),
                );
                let mut arguments = Vec::with_capacity(call.arguments.len());
                for argument in call.arguments.iter() {
                    let argument_ins = self.generate_ir_expression(
                        current_path,
                        completed_declarations,
                        lvt,
                        block,
                        argument,
                        None,
                    );
                    arguments.push(argument_ins);
                }
                Arc::new(Mutex::new(ir::Instruction::FunctionCall(Box::new(
                    ir::FunctionCall {
                        function,
                        arguments,
                    },
                ))))
            }
            ast::Expression::VariableReference(r) => {
                let name = r.name.clone();
                if let Some(path) = &r.path {
                    // this is a declaration to something in a module
                    let declaration = self.resolve(path.clone(), name.clone(), declaration_context);

                    Arc::new(Mutex::new(ir::Instruction::DeclarationReference(Box::new(
                        ir::DeclarationReference {
                            name: (Some(path.clone()), name),
                            declaration,
                        },
                    ))))
                } else {
                    // assume the symbol is in the module
                    if let Some((ins, generated)) = lvt.get(name.clone()) {
                        // this is a local variable
                        if !generated {
                            // return early if this instruction is already in the block
                            return ins;
                        }
                        ins
                    } else {
                        // search the current module declarations for it
                        let mut found_dec = None;
                        for declaration in completed_declarations.iter() {
                            if declaration.name() == name {
                                found_dec = Some(Arc::new(Mutex::new(
                                    ir::Instruction::DeclarationReference(Box::new(
                                        ir::DeclarationReference {
                                            name: (Some(current_path.clone()), name.clone()),
                                            declaration: ir::DeclarationContainer(Arc::new(Mutex::new(Some(
                                                declaration.clone(),
                                            )))),
                                        },
                                    )),
                                )));
                                break;
                            }
                        }
                        if let Some(found_dec) = found_dec {
                            found_dec
                        } else {
                            let declaration = self.resolve(current_path.clone(), name.clone(), declaration_context);
                            Arc::new(Mutex::new(ir::Instruction::DeclarationReference(Box::new(
                                ir::DeclarationReference {
                                    name: (Some(current_path.clone()), name),
                                    declaration,
                                },
                            ))))
                        }
                    }
                }
            }
            e => Arc::new(Mutex::new(ir::Instruction::Unreachable(format!(
                "UnhandledExpression({})",
                e.name()
            )))),
        };
        block.add_instruction(ins.clone());
        ins
    }
}

pub struct BlockBuilder {
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
        self.blocks
            .get_mut(self.current_block)
            .unwrap()
            .instructions
            .push(instruction);
    }
}

#[derive(Debug)]
pub struct LocalVariableTable {
    table: Vec<HashMap<String, Arc<Mutex<ir::Instruction>>>>,
    parameters: Vec<String>,
}

impl LocalVariableTable {
    pub fn new_with_params(parameters: Vec<String>) -> Self {
        let mut table = Vec::new();
        table.push(HashMap::new());
        Self { table, parameters }
    }

    pub fn new() -> Self {
        Self::new_with_params(vec![])
    }

    pub fn push_depth(&mut self) {
        self.table.push(HashMap::new());
    }

    pub fn pop_depth(&mut self) {
        self.table.pop();
    }

    pub fn get(&self, name: String) -> Option<(Arc<Mutex<ir::Instruction>>, bool)> {
        for x in self.table.iter().rev() {
            if let Some(instruction) = x.get(&name) {
                return Some((instruction.clone(), false));
            }
        }
        // check parameters
        for param in &self.parameters {
            if param == &name {
                return Some((
                    Arc::new(Mutex::new(ir::Instruction::GetParameter(Box::new(
                        ir::GetParameter { name },
                    )))),
                    true,
                ));
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
