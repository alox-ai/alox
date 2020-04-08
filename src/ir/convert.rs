use std::collections::HashMap;

use crate::ast;
use crate::ir;
use crate::util::Either;

impl ir::Compiler {
    pub fn generate_ir(&self, program: ast::Program) -> ir::Module {
        // args for module
        let name = program.file_name;
        let path = program.path;
        let current_path = &path.append(name.clone());
        let mut completed_declarations: Vec<ir::Declaration> = vec![];

        // go over each node and generate the ir
        for node in program.nodes {
            match node {
                ast::Node::Actor(a) => {
                    let mut fields = Vec::with_capacity(a.fields.len());
                    let mut functions = Vec::with_capacity(a.functions.len());
                    let mut behaviours = Vec::with_capacity(a.behaviours.len());

                    for field in a.fields {
                        let declaration = if let Some(type_path) = field.type_name {
                            ir::DeclarationId(format!("{}::{}", type_path.0.to_string(), type_path.1))
                        } else {
                            // TODO better errors
                            panic!("No type on field")
                        };
                        let variable = ir::Declaration::Variable(Box::new(ir::Variable {
                            name: field.name,
                            typ: declaration,
                        }));
                        fields.push(variable);
                    }

                    for f in a.functions {
                        let function = self.generate_ir_function(
                            current_path,
                            &completed_declarations,
                            &f,
                        );
                        functions.push(function);
                    }

                    for b in a.behaviours {
                        let behaviour = self.generate_ir_behaviour(
                            current_path,
                            &completed_declarations,
                            &b,
                        );
                        behaviours.push(behaviour);
                    }

                    let actor = ir::Actor {
                        name: a.name,
                        fields,
                        behaviours,
                        functions,
                    };
                    completed_declarations.push(ir::Declaration::Actor(Box::new(actor)));
                }
                ast::Node::Struct(s) => {
                    let mut fields = Vec::with_capacity(s.fields.len());
                    let mut traits = Vec::with_capacity(s.traits.len());
                    let mut functions = Vec::with_capacity(s.functions.len());

                    for field in s.fields {
                        let declaration = if let Some(type_path) = field.type_name {
                            ir::DeclarationId(format!("{}::{}", type_path.0.to_string(), type_path.1))
                        } else {
                            // TODO better errors
                            panic!("No type on field")
                        };
                        let variable = ir::Declaration::Variable(Box::new(ir::Variable {
                            name: field.name,
                            typ: declaration,
                        }));
                        fields.push(variable);
                    }

                    for f in s.functions {
                        let function = self.generate_ir_function(
                            current_path,
                            &completed_declarations,
                            &f,
                        );
                        functions.push(function);
                    }

                    let strct = ir::Struct {
                        name: s.name,
                        fields,
                        traits,
                        functions,
                    };
                    completed_declarations.push(ir::Declaration::Struct(Box::new(strct)));
                }
                ast::Node::Trait(_) => {}
                ast::Node::Function(f) => {
                    let function = self.generate_ir_function(
                        current_path,
                        &completed_declarations,
                        f.as_ref(),
                    );
                    completed_declarations.push(function);
                }
                ast::Node::VariableDeclaration(_) => {}
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
        declarations: &Vec<ir::Declaration>,
        b: &ast::Behaviour,
    ) -> ir::Declaration {
        let name = b.name.clone();

        // get the parameters from the function header
        let mut param_names = vec![];
        for (name, _dec) in &b.arguments {
            param_names.push(name.clone());
        }

        let mut block_builder = BlockBuilder::new();
        let mut lvt = &mut LocalVariableTable::new_with_params(param_names);

        {
            let block_builder = &mut block_builder;
            for statement in b.statements.iter() {
                self.generate_ir_statement(
                    current_path,
                    declarations,
                    lvt,
                    block_builder,
                    statement,
                );
            }
        }

        let mut blocks = Vec::with_capacity(block_builder.blocks.len());
        for block in block_builder.blocks {
            if block.instructions.len() > 0 {
                blocks.push(block.clone());
            }
        }

        let mut arguments = Vec::with_capacity(b.arguments.len());
        for (name, type_path) in b.arguments.iter() {
            let declaration_id = ir::DeclarationId(format!("{}::{}", type_path.0.to_string(), type_path.1));
            arguments.push((name.clone(), declaration_id));
        }

        ir::Declaration::Behaviour(Box::new(ir::Behaviour {
            name,
            arguments,
            blocks,
        }))
    }

    pub fn generate_ir_function(
        &self,
        current_path: &ast::Path,
        declarations: &Vec<ir::Declaration>,
        f: &ast::Function,
    ) -> ir::Declaration {
        let name = f.name.clone();

        // get the parameters from the function header
        let mut param_names = vec![];
        for (name, _dec) in &f.arguments {
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

        let mut blocks = Vec::with_capacity(block_builder.blocks.len());
        for block in block_builder.blocks {
            if block.instructions.len() > 0 {
                blocks.push(block.clone());
            }
        }

        let mut arguments = Vec::with_capacity(f.arguments.len());
        for (name, type_path) in f.arguments.iter() {
            let declaration_id = ir::DeclarationId(format!("{}::{}", type_path.0.to_string(), type_path.1));
            arguments.push((name.clone(), declaration_id));
        }
        let return_type = ir::DeclarationId(format!("{}::{}", f.return_type.0.to_string(), f.return_type.1));

        ir::Declaration::Function(Box::new(ir::Function {
            name,
            arguments,
            return_type,
            blocks,
        }))
    }

    pub fn generate_ir_statement(
        &self,
        current_path: &ast::Path,
        completed_declarations: &Vec<ir::Declaration>,
        mut lvt: &mut LocalVariableTable,
        mut block_builder: &mut BlockBuilder,
        statement: &ast::Statement,
    ) {
        match statement {
            ast::Statement::VariableDeclaration(d) => {
                if let Some(exp) = &d.initial_expression {
                    let current_block = block_builder.current_block();
                    let expr_ins = self.generate_ir_expression(
                        current_path,
                        completed_declarations,
                        lvt,
                        current_block,
                        exp,
                    );
                    lvt.set(d.name.clone(), expr_ins);
                } else {
                    // lvt.set(d.name.clone(), &ir::Instruction::Unreachable("MissingInitialExpression".to_string()));
                    panic!("variable declaration missing initial expression");
                }
            }
            ast::Statement::Return(r) => {
                let expr_ins = self.generate_ir_expression(
                    current_path,
                    completed_declarations,
                    lvt,
                    block_builder.current_block(),
                    &r.expression,
                );
                let return_ins = ir::Instruction::Return(Box::new(ir::Return {
                    instruction: expr_ins,
                }));
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
                );
                let mut arguments = Vec::with_capacity(call.arguments.len());
                for argument in call.arguments.iter() {
                    let argument_ins = self.generate_ir_expression(
                        current_path,
                        completed_declarations,
                        lvt,
                        block_builder.current_block(),
                        argument,
                    );
                    arguments.push(argument_ins);
                }
                let call_ins = ir::Instruction::FunctionCall(Box::new(
                    ir::FunctionCall {
                        function,
                        arguments,
                    },
                ));
                block_builder.add_instruction(call_ins);
            }
            ast::Statement::If(if_statement) => {
                self.generate_ir_if_statement(current_path, completed_declarations, &mut lvt, &mut block_builder, &if_statement)
            }
        }
    }

    fn generate_ir_if_statement(
        &self,
        current_path: &ast::Path,
        completed_declarations: &Vec<ir::Declaration>,
        mut lvt: &mut LocalVariableTable,
        mut block_builder: &mut BlockBuilder,
        if_statement: &Box<ast::IfStatement>,
    ) {
        let current_block_id: ir::BlockId = ir::BlockId(block_builder.current_block);

        let condition = self.generate_ir_expression(
            current_path,
            completed_declarations,
            lvt,
            block_builder.current_block(),
            &if_statement.condition,
        );

        let condition_ins = block_builder.current_block().get_instruction(condition);

        let literal_condition: Option<bool> = match condition_ins {
            ir::Instruction::BooleanLiteral(ref b) => Some((*b).0),
            _ => None,
        };

        block_builder.create_block();
        let true_block_id: ir::BlockId = ir::BlockId(block_builder.current_block);

        for statement in if_statement.block.iter() {
            self.generate_ir_statement(
                current_path,
                completed_declarations,
                &mut lvt,
                &mut block_builder,
                statement,
            );
        }

        block_builder.create_block();
        let false_block_id: ir::BlockId = ir::BlockId(block_builder.current_block);

        if let Some(elseif) = &if_statement.elseif {
            self.generate_ir_if_statement(
                current_path,
                completed_declarations,
                &mut lvt,
                &mut block_builder,
                elseif,
            );
        }

        block_builder.create_block();
        let merge_block_id: ir::BlockId = ir::BlockId(block_builder.current_block);

        if let Some(literal) = literal_condition {
            if literal {
                let jump = ir::Instruction::Jump(Box::new(ir::Jump { block: true_block_id }));
                block_builder.blocks.get_mut(current_block_id.0).expect("uh we just added this block?").add_instruction(jump, self);
            } else {
                let jump = ir::Instruction::Jump(Box::new(ir::Jump { block: false_block_id }));
                block_builder.blocks.get_mut(current_block_id.0).expect("uh we just added this block?").add_instruction(jump, self);
            }
        } else {
            let branch = ir::Instruction::Branch(Box::new(ir::Branch {
                condition,
                true_block: true_block_id,
                false_block: false_block_id,
            }));
            block_builder.blocks.get_mut(current_block_id.0).expect("uh we just added this block?").add_instruction(branch, self);
        }

        let jump = ir::Instruction::Jump(Box::new(ir::Jump { block: merge_block_id }));
        if let Some(literal) = literal_condition {
            if literal {
                block_builder.blocks.get_mut(true_block_id.0).expect("uh we just added this block?").add_instruction(jump, self);
            } else {
                block_builder.blocks.get_mut(false_block_id.0).expect("uh we just added this block?").add_instruction(jump, self);
            }
        }
    }

    pub fn generate_ir_expression(
        &self,
        current_path: &ast::Path,
        completed_declarations: &Vec<ir::Declaration>,
        lvt: &mut LocalVariableTable,
        block: &mut ir::Block,
        expression: &ast::Expression,
    ) -> ir::InstructionId {
        let block_copy: &mut ir::Block = unsafe { &mut *(block as *mut ir::Block) };
        let ins = match expression {
            ast::Expression::BooleanLiteral(b) =>
                ir::Instruction::BooleanLiteral(Box::new(ir::BooleanLiteral(b.as_ref().0))),
            ast::Expression::IntegerLiteral(i) =>
                ir::Instruction::IntegerLiteral(Box::new(ir::IntegerLiteral(i.as_ref().0))),
            ast::Expression::FunctionCall(call) => {
                let block_copy2: &mut ir::Block = unsafe { &mut *(block_copy as *mut ir::Block) };
                let function = self.generate_ir_expression(
                    current_path,
                    completed_declarations,
                    lvt,
                    block_copy,
                    &call.function,
                );
                let mut arguments = Vec::with_capacity(call.arguments.len());
                for argument in call.arguments.iter() {
                    let block_copy3: &mut ir::Block = unsafe { &mut *(block_copy2 as *mut ir::Block) };
                    let argument_ins = self.generate_ir_expression(
                        current_path,
                        completed_declarations,
                        lvt,
                        block_copy3,
                        argument,
                    );
                    arguments.push(argument_ins);
                }
                ir::Instruction::FunctionCall(Box::new(
                    ir::FunctionCall {
                        function,
                        arguments,
                    },
                ))
            }
            ast::Expression::VariableReference(r) => {
                let name = r.name.clone();
                if let Some(path) = &r.path {
                    // this is a declaration to something in a module
                    let declaration_id = ir::DeclarationId(format!("{}::{}", path.to_string(), name.clone()));
                    ir::Instruction::DeclarationReference(Box::new(
                        ir::DeclarationReference {
                            name: (Some(path.clone()), name),
                            declaration: declaration_id,
                        },
                    ))
                } else {
                    // assume the symbol is in the module
                    let result: Option<Either<ir::InstructionId, ir::Instruction>> = lvt.get(name.clone());
                    if let Some(result) = result {
                        match result {
                            Either::Left(reference) => {
                                // return early if this instruction is already in the block
                                return reference;
                            }
                            Either::Right(generated) => {
                                generated
                            }
                        }
                    } else {
                        // search the current module declarations for it
                        let mut found_dec = None;
                        for declaration in completed_declarations.iter() {
                            if declaration.name() == name {
                                let declaration_id = ir::DeclarationId(format!("{}::{}", current_path.to_string(), name.clone()));
                                found_dec = Some(ir::Instruction::DeclarationReference(Box::new(
                                    ir::DeclarationReference {
                                        name: (Some(current_path.clone()), name.clone()),
                                        declaration: declaration_id,
                                    },
                                )));
                                break;
                            }
                        }
                        if let Some(found_dec) = found_dec {
                            found_dec
                        } else {
                            let declaration = ir::DeclarationId(format!("{}::{}", current_path.to_string(), name.clone()));
                            ir::Instruction::DeclarationReference(Box::new(
                                ir::DeclarationReference {
                                    name: (Some(current_path.clone()), name),
                                    declaration,
                                },
                            ))
                        }
                    }
                }
            }
            e => ir::Instruction::Unreachable(format!(
                "UnhandledExpression({})",
                e.name()
            )),
        };
        block.add_instruction(ins, self)
    }
}

pub struct BlockBuilder {
    blocks: Vec<ir::Block>,
    current_block: usize,
}

impl BlockBuilder {
    pub fn new() -> Self {
        let mut blocks = Vec::new();
        blocks.push(ir::Block::new(0));
        Self {
            blocks,
            current_block: 0,
        }
    }

    pub fn current_block(&mut self) -> &mut ir::Block {
        let result: &mut ir::Block = self.blocks.get_mut(self.current_block).unwrap();
        // let mut x = unsafe { &mut *(result as *mut ir::Block) };
        result
    }

    pub fn create_block(&mut self) -> &mut ir::Block {
        self.current_block = self.blocks.len();
        self.blocks.push(ir::Block::new(self.current_block));
        self.current_block()
    }

    pub fn add_instruction(&mut self, instruction: ir::Instruction) {
        self.blocks
            .get_mut(self.current_block)
            .unwrap()
            .instructions
            .push(instruction);
    }

    pub fn block_id(&self, block: &ir::Block) -> ir::BlockId {
        let index = self.blocks.iter().position(|b| b as *const ir::Block == block as *const ir::Block).unwrap();
        ir::BlockId(index)
    }
}

#[derive(Debug)]
pub struct LocalVariableTable {
    table: Vec<HashMap<String, ir::InstructionId>>,
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

    pub fn get(&self, name: String) -> Option<Either<ir::InstructionId, ir::Instruction>> {
        for x in self.table.iter().rev() {
            if let Some(instruction) = x.get(&name) {
                return Some(Either::Left(*instruction));
            }
        }
        // check parameters
        for param in &self.parameters {
            if param == &name {
                return Some(Either::Right(ir::Instruction::GetParameter(Box::new(
                    ir::GetParameter { name },
                ))));
            }
        }
        None
    }

    pub fn set(&mut self, name: String, instruction: ir::InstructionId) {
        if let Some(map) = self.table.last_mut() {
            map.insert(name, instruction);
        }
    }
}
