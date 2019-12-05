use crate::ir::*;

use cranelift_codegen::ir::types::*;
use cranelift_codegen::ir::{AbiParam, ExternalName, Function, InstBuilder, Signature};
use cranelift_codegen::isa::CallConv;
use cranelift_codegen::settings;
use cranelift_codegen::verifier::verify_function;
use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext, Variable};
use crate::ir::types::PrimitiveType;
use crate::ir::types::Type::*;
use crate::ir;
use std::sync::{Arc, Mutex};
use crate::util::Either;
use std::collections::HashMap;

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
                    _ => None,
                }
            }
            PrimitiveType::Float(f) => {
                match f {
                    32 => Some(F32),
                    64 => Some(F64),
                    _ => None
                }
            }
            PrimitiveType::Void => None,
            PrimitiveType::NoReturn => None,
        }
    }

    pub fn convert_type(&self, typ: Box<crate::ir::types::Type>) -> Option<Type> {
        match *typ {
            Primitive(p) => self.convert_primitive_type(p),
            _ => { panic!("can't convert type for CL IR"); /* TODO */ }
        }
    }

    pub fn convert_module(&self, module: &Module) {
        for declaration in &module.declarations {
            self.convert_declaration(&*declaration.read().unwrap(), None);
        }
    }

    fn convert_declaration(&self, dec: &Declaration, context: Option<Either<&Box<ir::Struct>, &Box<ir::Actor>>>) {
        match dec {
            Declaration::Function(ref function) => {
                self.convert_function(Either::Left(function), context);
            }
            Declaration::Behaviour(ref behaviour) => {
                self.convert_function(Either::Right(behaviour), context);
            }
            Declaration::Struct(ref struc) => {
                let guard = struc.functions.read().unwrap();
                for dec in guard.iter() {
                    let guard = dec.0.lock().unwrap();
                    if let Some(ref dec) = *guard {
                        self.convert_declaration(&*dec.read().unwrap(), Some(Either::Left(struc)));
                    }
                }
            }
            Declaration::Actor(ref actor) => {
                let guard = actor.functions.read().unwrap();
                for dec in guard.iter() {
                    let guard = dec.0.lock().unwrap();
                    if let Some(ref dec) = *guard {
                        self.convert_declaration(&*dec.read().unwrap(), Some(Either::Right(actor)));
                    }
                }

                let guard = actor.behaviours.read().unwrap();
                for dec in guard.iter() {
                    let guard = dec.0.lock().unwrap();
                    if let Some(ref dec) = *guard {
                        self.convert_declaration(&*dec.read().unwrap(), Some(Either::Right(actor)));
                    }
                }
            }
            _ => { /*TODO*/ }
        }
    }

    fn convert_function(&self, function: Either<&Box<ir::Function>, &Box<ir::Behaviour>>, context: Option<Either<&Box<ir::Struct>, &Box<ir::Actor>>>) -> Function {
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

        let mut func = Function::with_name_signature(ExternalName::testcase(name), sig);
        let blocks = match function {
            Either::Left(function) => &function.blocks,
            Either::Right(behaviour) => &behaviour.blocks,
        };
        self.convert_blocks(&mut func, blocks);
        func
    }

    fn convert_blocks(&self, mut func: &mut Function, blocks: &Vec<Arc<Mutex<Block>>>) {
        let mut fn_builder_ctx = FunctionBuilderContext::new();
        let mut builder = FunctionBuilder::new(&mut func, &mut fn_builder_ctx);

        for block in blocks {
            let ebb = Arc::new(builder.create_ebb());
            let block = block.lock().unwrap();

            for instruction in &block.instructions {
                let instruction = instruction.lock().unwrap();
                match *instruction {
                    _ => { /*TODO*/ }
                }
            }
        }
        builder.finalize();

        println!("{}", func.display(None));
    }
}
