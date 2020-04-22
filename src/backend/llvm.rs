use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_uint};
use std::path::Path;
use std::sync::Arc;

use llvm_sys::*;
use llvm_sys::core::*;
use llvm_sys::error::*;
use llvm_sys::prelude::*;
use llvm_sys::target::*;
use llvm_sys::target_machine::*;

use crate::ir::*;
use crate::ir::types::*;
use std::ops::Deref;
use std::ptr;

pub struct LLVMBackend<'compiler> {
    compiler: &'compiler Compiler,
    module: &'compiler Module,
    context: LLVMContextRef,
    builder: LLVMBuilderRef,
    llvm_module: LLVMModuleRef,
    type_cache: HashMap<Type, LLVMTypeRef>,
    dec_cache: HashMap<DeclarationId, (LLVMValueRef, LLVMTypeRef)>,
}

impl<'c> LLVMBackend<'c> {
    pub fn new(compiler: &'c Compiler, module: &'c Module) -> Self {
        unsafe {
            LLVMInitializeX86TargetInfo();
            LLVMInitializeX86Target();
            LLVMInitializeX86TargetMC();
            LLVMInitializeX86AsmPrinter();
            let context = LLVMContextCreate();
            let module_name = CString::new(module.name.clone()).unwrap();
            let llvm_module: LLVMModuleRef = LLVMModuleCreateWithNameInContext(module_name.as_ptr(), context);
            let builder: LLVMBuilderRef = LLVMCreateBuilder();

            /*
            let target_triple = LLVMGetDefaultTargetTriple();
            let mut target = ptr::null_mut();
            let mut error = ptr::null_mut();
            LLVMGetTargetFromTriple(target_triple, target, error);
            LLVMDisposeErrorMessage(*error);
            let target_machine = LLVMCreateTargetMachine(*target, target_triple, cstr(""), cstr(""), LLVMCodeGenOptLevel::LLVMCodeGenLevelNone, LLVMRelocMode::LLVMRelocDefault, LLVMCodeModel::LLVMCodeModelDefault);
            LLVMDisposeMessage(target_triple);
            */

            Self {
                compiler,
                module,
                context,
                builder,
                llvm_module,
                type_cache: HashMap::new(),
                dec_cache: HashMap::new(),
            }
        }
    }

    unsafe fn convert_type(&mut self, typ: Type) -> LLVMTypeRef {
        if let Some(typ) = self.type_cache.get(&typ) {
            return *typ;
        }
        let llvm_type: LLVMTypeRef = match typ.clone() {
            Type::Primitive(p) => {
                match p {
                    PrimitiveType::Int(b) => {
                        LLVMIntTypeInContext(self.context, b as u32)
                    }
                    PrimitiveType::Float(b) => {
                        match b {
                            16 => LLVMHalfTypeInContext(self.context),
                            32 => LLVMFloatTypeInContext(self.context),
                            64 => LLVMDoubleTypeInContext(self.context),
                            80 => LLVMX86FP80TypeInContext(self.context), // TODO: research this
                            128 => LLVMFP128TypeInContext(self.context),
                            b => panic!(format!("couldn't convert float type {:?}", b)),
                        }
                    }
                    PrimitiveType::Bool => {
                        LLVMIntTypeInContext(self.context, 1)
                    }
                    p => panic!(format!("couldn't convert primitive type {:?}", p))
                }
            }
            Type::Function(f) => {
                let mut args = Vec::with_capacity(f.arguments.len());
                for arg in f.arguments.iter() {
                    args.push(self.convert_type(arg.deref().clone()));
                }
                let result_type = self.convert_type(f.result.deref().clone());
                LLVMFunctionType(result_type, args.as_mut_ptr(), args.len() as c_uint, 0)
            }
            t => panic!(format!("couldn't convert type {:?}", t))
        };

        self.type_cache.insert(typ, llvm_type);
        llvm_type
    }

    pub fn process(&mut self) {
        // process "headers" of declarations first
        for declaration in self.module.declarations.iter() {
            self.process_declaration_header(declaration);
        }
        for declaration in self.module.declarations.iter() {
            self.process_declaration(declaration);
        }
    }

    /// Generate anything needed to refer to a declaration out of order, like
    /// functions calling later defined functions needing a declaration
    fn process_declaration_header(&mut self, declaration: &Declaration) {
        let declaration_id = DeclarationId::from(Some(self.module), declaration);
        match declaration {
            Declaration::Function(ref function) => {
                let function_name = CString::new(function.name.clone()).unwrap();
                self.process_function_header(declaration_id, function_name, function.as_ref());
            }
            // TODO: cache struct types?
            _ => {}
        }
    }

    /// Declares the function in the LLVM module without creating a body for it.
    fn process_function_header(
        &mut self,
        declaration_id: DeclarationId,
        function_name: CString,
        function: &Function,
    ) {
        unsafe {
            let function_type = function.get_type(self.compiler);
            let llvm_function_type: LLVMTypeRef = self.convert_type(function_type.deref().clone());
            let llvm_function: LLVMValueRef = LLVMAddFunction(self.llvm_module, function_name.as_ptr(), llvm_function_type);
            self.dec_cache.insert(declaration_id, (llvm_function, llvm_function_type));
        }
    }

    fn process_declaration(&mut self, declaration: &Declaration) {
        let declaration_id = DeclarationId::from(Some(self.module), declaration);
        match declaration {
            Declaration::Function(ref function) => {
                let function_name = CString::new(function.name.clone()).unwrap();
                self.process_function_body(declaration_id, function_name, function.as_ref());
            }
            _ => {}
        }
    }

    fn process_function_body(
        &mut self,
        declaration_id: DeclarationId,
        function_name: CString,
        function: &Function,
    ) {
        unsafe {
            let (llvm_function, llvm_function_type) = self.dec_cache.get(&declaration_id).unwrap().clone();
            LLVMSetFunctionCallConv(llvm_function, LLVMCallConv::LLVMCCallConv as u32);
            self.process_blocks(declaration_id, function_name, function, llvm_function, function.blocks.as_slice());
        }
    }

    fn process_blocks(
        &mut self,
        declaration_id: DeclarationId,
        function_name: CString,
        function: &Function,
        llvm_function: LLVMValueRef,
        blocks: &[Block],
    ) {
        let mut block_map: HashMap<usize, LLVMBasicBlockRef> = HashMap::new();
        let mut instruction_map: HashMap<usize, LLVMValueRef> = HashMap::new();

        for (block_id, _) in blocks.iter().enumerate() {
            let block_name = CString::new(format!("block{}", block_id)).unwrap();
            let llvm_block = unsafe { LLVMAppendBasicBlock(llvm_function, block_name.as_ptr()) };
            block_map.insert(block_id, llvm_block);
        }

        for (block_id, block) in blocks.iter().enumerate() {
            let llvm_block = *block_map.get(&block_id).unwrap();
            unsafe {
                LLVMPositionBuilderAtEnd(self.builder, llvm_block);

                for (instruction_id, instruction) in block.instructions.iter().enumerate() {
                    let instruction_id = instruction_id + block.ins_start_offset;
                    let instruction_name = CString::new(format!("ins{}", instruction_id)).unwrap();

                    match *instruction {
                        Instruction::IntegerLiteral(ref i) => {
                            let int_type = self.convert_type(Type::Primitive(PrimitiveType::Int(32)));
                            let value = LLVMConstInt(int_type, i.0 as u64, 0);
                            instruction_map.insert(instruction_id, value);
                        }
                        Instruction::BooleanLiteral(ref b) => {
                            let bool_type = self.convert_type(Type::Primitive(PrimitiveType::Bool));
                            let value = LLVMConstInt(bool_type, if b.0 { 1 } else { 0 }, 0);
                            instruction_map.insert(instruction_id, value);
                        }
                        Instruction::Alloca(ref a) => {
                            let typ = block.get_instruction(a.reference_ins).get_type(self.compiler, block);
                            let llvm_type = self.convert_type(typ.deref().clone());
                            let value = LLVMBuildAlloca(self.builder, llvm_type, instruction_name.as_ptr());
                            instruction_map.insert(instruction_id, value);
                        }
                        Instruction::Store(ref s) => {
                            let ptr = *instruction_map.get(&s.ptr.0).unwrap();
                            let val = *instruction_map.get(&s.value.0).unwrap();
                            let value = LLVMBuildStore(self.builder, val, ptr);
                            instruction_map.insert(instruction_id, value);
                        }
                        Instruction::Load(ref l) => {
                            // let typ = instruction.get_type_with_context(self.compiler, block, function);
                            // let typ = block.get_instruction(l.reference_ins).get_type(self.compiler, block);
                            // let llvm_type = self.convert_type(typ.deref().clone());
                            let ptr = *instruction_map.get(&l.ptr.0).unwrap();
                            let value = LLVMBuildLoad(self.builder, ptr, instruction_name.as_ptr());
                            instruction_map.insert(instruction_id, value);
                        }
                        Instruction::GetParameter(ref g) => {
                            for (i, (name, _)) in function.arguments.iter().enumerate() {
                                if name == &g.name {
                                    let value = LLVMGetParam(llvm_function, i as c_uint);
                                    instruction_map.insert(instruction_id, value);
                                    break;
                                }
                            }
                        }
                        Instruction::Jump(ref j) => {
                            let to_llvm_block = block_map.get(&j.block.0).unwrap();
                            LLVMBuildBr(self.builder, *to_llvm_block);
                        }
                        Instruction::Branch(ref b) => {
                            let cond = *instruction_map.get(&b.condition.0).unwrap();
                            let true_block = *block_map.get(&b.true_block.0).unwrap();
                            let false_block = *block_map.get(&b.false_block.0).unwrap();
                            LLVMBuildCondBr(self.builder, cond, true_block, false_block);
                        }
                        Instruction::Return(ref r) => {
                            let value = *instruction_map.get(&r.instruction.0)
                                .expect(&format!("couldn't find instruction: {}", &r.instruction.0));
                            LLVMBuildRet(self.builder, value);
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    pub fn dump(&self) {
        unsafe {
            LLVMDumpModule(self.llvm_module);
        }
    }

    pub fn emit<S: AsRef<str>>(&mut self, path: S) {
        unsafe {
            let error = ptr::null_mut();
            let cpath = CString::new(path.as_ref()).unwrap();
            LLVMPrintModuleToFile(self.llvm_module, cpath.as_ptr(), error);
            LLVMDisposeErrorMessage(*error);
        }
    }
}

impl Drop for LLVMBackend<'_> {
    fn drop(&mut self) {
        unsafe {
            LLVMDisposeModule(self.llvm_module);
            LLVMDisposeBuilder(self.builder);
            LLVMContextDispose(self.context);
        }
    }
}