use crate::ir::*;

use cranelift_codegen::ir::types::*;
use cranelift_codegen::ir::{AbiParam, ExternalName, Function as CLFunction, InstBuilder, Signature, InstBuilderBase, Value, Ebb};
use cranelift_codegen::isa::CallConv;
use cranelift_codegen::settings;
use cranelift_codegen::verifier::verify_function;
use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext, Variable};
use cranelift_faerie::{FaerieBackend, FaerieBuilder, FaerieTrapCollection};
use cranelift_module::{self, DataId, FuncId, Linkage, Module as CraneliftModule};
use crate::ir::types::PrimitiveType;
use crate::ir::types::Type::*;
use crate::ir;
use std::sync::{Arc, Mutex};
use crate::util::Either;
use std::collections::HashMap;
use std::borrow::Borrow;

pub struct CraneLiftBackend {}

impl CraneLiftBackend {
    pub fn new() -> CraneLiftBackend {
        CraneLiftBackend {}
    }

    pub fn convert_primitive_type(&self, primitive: PrimitiveType) -> Option<Type> {
        match primitive {
            PrimitiveType::Bool => Some(B8),
            PrimitiveType::Int(i) => {
                match i {
                    16 => Some(I16),
                    32 => Some(I32),
                    64 => Some(I64),
                    128 => Some(I128),
                    255 => Some(I64),
                    i => panic!("unsupported int type {} in cranelift backend", i),
                }
            }
            PrimitiveType::Float(f) => {
                match f {
                    32 => Some(F32),
                    64 => Some(F64),
                    255 => Some(F64),
                    f => panic!("unsupported float type {} in cranelift backend", f),
                }
            }
            PrimitiveType::Void => None,
            PrimitiveType::NoReturn => None,
        }
    }

    pub fn convert_type(&self, typ: Box<crate::ir::types::Type>) -> Option<Type> {
        match *typ {
            Primitive(p) => self.convert_primitive_type(p),
            t => { panic!(format!("can't convert type for CL IR: {:#?}", t)); /* TODO */ }
        }
    }

    /// todo: not return string
    pub fn convert_module(&self, compiler: &Compiler, module: &Module) -> String {
        let mut buffer = String::new();
        for declaration in &module.declarations {
            buffer.push_str(&self.convert_declaration(compiler, declaration, None));
            buffer.push('\n');
        }
        buffer
    }

    /// todo: not return string
    fn convert_declaration(
        &self,
        compiler: &Compiler,
        dec: &Declaration,
        context: Option<Either<&Box<ir::Struct>, &Box<ir::Actor>>>,
    ) -> String {
        match dec {
            Declaration::Function(ref function) => {
                self.convert_function(compiler, Either::Left(function), context).display(None).to_string()
            }
            Declaration::Behaviour(ref behaviour) => {
                self.convert_function(compiler, Either::Right(behaviour), context).display(None).to_string()
            }
            Declaration::Struct(ref struc) => {
                let mut buffer = String::new();
                for dec in struc.functions.iter() {
                    let dec = self.convert_declaration(compiler, dec, Some(Either::Left(struc)));
                    buffer.push_str(&dec);
                    buffer.push('\n');
                }
                buffer
            }
            Declaration::Actor(ref actor) => {
                let mut buffer = String::new();
                for dec in actor.functions.iter() {
                    let dec = self.convert_declaration(compiler, dec, Some(Either::Right(actor)));
                    buffer.push_str(&dec);
                    buffer.push('\n');
                }

                for dec in actor.behaviours.iter() {
                    let dec = self.convert_declaration(compiler, dec, Some(Either::Right(actor)));
                    buffer.push_str(&dec);
                    buffer.push('\n');
                }
                buffer
            }
            _ => { "".to_string() }
        }
    }

    fn convert_function(
        &self,
        compiler: &Compiler,
        function: Either<&Box<ir::Function>, &Box<ir::Behaviour>>,
        context: Option<Either<&Box<ir::Struct>, &Box<ir::Actor>>>,
    ) -> CLFunction {
        let mut sig = Signature::new(CallConv::SystemV);
        // convert return type
        if let Either::Left(function) = function {
            if let Some(typ) = self.convert_type(function.return_type.get_type()) {
                sig.returns.push(AbiParam::new(typ));
            }
        }

        // convert args
        let args = match function {
            Either::Left(function) => &function.arguments,
            Either::Right(behaviour) => &behaviour.arguments,
        };

        for (_name, arg) in args {
            if let Some(typ) = self.convert_type(arg.get_type()) {
                sig.params.push(AbiParam::new(typ));
            }
        }

        // build name from context
        let name = if let Some(context) = context {
            let mut name = match context {
                Either::Left(struc) => struc.name.clone(),
                Either::Right(actor) => actor.name.clone(),
            };
            name.push('_');
            let n = match function {
                Either::Left(function) => &function.name,
                Either::Right(behaviour) => &behaviour.name,
            };
            name.push_str(n);
            name
        } else {
            match function {
                Either::Left(function) => function.name.clone(),
                Either::Right(behaviour) => behaviour.name.clone(),
            }
        };

        let mut func = CLFunction::with_name_signature(ExternalName::testcase(name), sig);
        let blocks = match function {
            Either::Left(function) => &function.blocks,
            Either::Right(behaviour) => &behaviour.blocks,
        };
        self.convert_blocks(compiler, &mut func, blocks);
        func
    }

    fn convert_blocks(&self, compiler: &Compiler, mut func: &mut CLFunction, blocks: &Vec<Block>) {
        let mut fn_builder_ctx = FunctionBuilderContext::new();
        let mut builder = FunctionBuilder::new(&mut func, &mut fn_builder_ctx);

        let mut value_map: HashMap<usize, Value> = HashMap::new();
        let mut block_map: HashMap<usize, Ebb> = HashMap::new();
        // create an ebb for every block
        for (block_id, _) in blocks.iter().enumerate() {
            let ebb = builder.create_ebb();
            block_map.insert(block_id, ebb);
        }
        for (block_id, block) in blocks.iter().enumerate() {
            let current_ebb = block_map.get(&block_id).unwrap();
            builder.switch_to_block(*current_ebb);
            for (instruction_id, instruction) in block.instructions.iter().enumerate() {
                match *instruction {
                    Instruction::IntegerLiteral(ref i) => {
                        let typ = self.convert_type(instruction.get_type(compiler, block)).expect("int literal should be an int type");
                        let value: Value = builder.ins().iconst(typ, i.0);
                        value_map.insert(instruction_id, value);
                    }
                    Instruction::BooleanLiteral(ref b) => {
                        let typ = self.convert_type(instruction.get_type(compiler, block)).expect("int literal should be an int type");
                        let value: Value = builder.ins().bconst(typ, b.0);
                        value_map.insert(instruction_id, value);
                    }
                    // Instruction::GetParameter(ref param) => {
                    //     param.name
                    // }
                    Instruction::Jump(ref jump) => {
                        let to_block_id = jump.block.0;
                        let to_block_ebb = block_map.get(&to_block_id).expect("referred to block not in function");
                        builder.ins().jump(*to_block_ebb, &[]);
                    }
                    Instruction::Branch(ref branch) => {
                        let condition_value = value_map.get(&(branch.condition.0))
                            .expect("missing condition value");
                        let true_ebb = block_map.get(&(branch.true_block.0))
                            .expect("referred to block not in function");
                        let false_ebb = block_map.get(&(branch.false_block.0))
                            .expect("referred to block not in function");
                        builder.ins().brnz(*condition_value, *true_ebb, &[]);
                        builder.ins().jump(*false_ebb, &[]);
                    }
                    Instruction::Return(ref ret) => {
                        // ref of the value we're going to return
                        let ret_value_id = ret.instruction.0;
                        if let Some(value) = value_map.get(&ret_value_id) {
                            builder.ins().return_(&[value.clone()]);
                        }
                    }
                    _ => { /*TODO*/ }
                }
            }
        }
        // TODO: uncomment when done with instructions
        // builder.finalize();
    }
}
