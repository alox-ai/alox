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
            let mut target = 0 as *mut _;
            let mut error = 0 as *mut _;
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
        // convert declarations
        for declaration in self.module.declarations.iter() {
            self.process_declaration(declaration);
        }
    }

    fn process_declaration(&mut self, declaration: &Declaration) {
        let declaration_id = DeclarationId::from(Some(self.module), declaration);
        match declaration {
            Declaration::Function(ref function) => {
                let function_name = CString::new(function.name.clone()).unwrap();
                self.process_function(declaration_id, function_name, function.as_ref());
            }
            _ => {}
        }
    }

    fn process_function(
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

    pub fn dump(&self) {
        unsafe {
            LLVMDumpModule(self.llvm_module);
        }
    }

    pub fn emit<S: AsRef<str>>(&mut self, path: S) {
        unsafe {
            let error = 0 as *mut _;
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