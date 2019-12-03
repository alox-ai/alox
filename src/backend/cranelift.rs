use crate::ir::*;

use cranelift_codegen::entity::EntityRef;
use cranelift_codegen::ir::types::*;
use cranelift_codegen::ir::{AbiParam, ExternalName, Function, InstBuilder, Signature};
use cranelift_codegen::isa::CallConv;
use cranelift_codegen::settings;
use cranelift_codegen::verifier::verify_function;
use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext, Variable};

pub struct CraneLiftBackend {}

impl CraneLiftBackend {
    pub fn new() -> CraneLiftBackend {
        CraneLiftBackend {}
    }

    pub fn convert_type(&self, typ: Box<dyn crate::ir::types::Type>) -> Type {
        return B1; // TODO
    }

    pub fn compile_module(&self, module: &Module) {
        for declaration in &module.declarations {
            match **declaration {
                Declaration::Function(ref function) => {
                    let mut sig = Signature::new(CallConv::SystemV);
                    // convert return type
                    sig.returns.push(AbiParam::new(self.convert_type(function.return_type.get_type())));
                    // convert args
                    for (name, arg) in &function.arguments {
                        sig.params.push(AbiParam::new(self.convert_type(arg.get_type())));
                    }
                    // create function
                    let mut fn_builder_ctx = FunctionBuilderContext::new();
                    let mut func = Function::with_name_signature(ExternalName::user(0, 0), sig);

                    {
                        let mut builder = FunctionBuilder::new(&mut func, &mut fn_builder_ctx);

                        for block in &function.blocks {
                            let block = block.lock().unwrap();
                            let ebb = builder.create_ebb();
                            builder.seal_block(ebb);

                            for instruction in &block.instructions {
                                let instruction = instruction.lock().unwrap();
                                match instruction {
                                    _ => { /*TODO*/ }
                                }
                            }
                        }
                        builder.finalize();

                        println!("{}", func.display(None));
                    }
                }
                _ => { /*TODO*/ }
            }
        }
    }
}
