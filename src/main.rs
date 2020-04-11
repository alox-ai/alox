extern crate codespan;
extern crate codespan_reporting;
extern crate cranelift_codegen;
extern crate cranelift_faerie;
extern crate cranelift_frontend;
extern crate cranelift_module;
extern crate cuda;
#[macro_use]
extern crate lalrpop_util;
#[macro_use]
extern crate lazy_static;
extern crate llvm_sys;
extern crate rspirv;
extern crate spirv_headers as spirv;

use crate::backend::cranelift::CraneLiftBackend;
use crate::ir::debug::PrintMode;
use crate::ir::pass::{DeadBranchRemovalPass, Pass};
use crate::ir::Compiler;
use crate::backend::llvm::LLVMBackend;

pub mod diagnostic;
pub mod parser;
pub mod ast;
pub mod ir;
pub mod backend;
pub mod util;

fn main() {
    let test_file = "test.alox";
    let test_source = "\
    fun test(a: Int32, b: fun Int32): Int32 {
        var x = 1
        if false {
            return 1
        }
        return x
    }".to_string();
    let mut parser = parser::Parser::new();
    let parsed_program = parser.parse(ast::Path::of("test"), test_file.to_string(), test_source);
    parser.diagnostics.emit_errors();
    println!("{}", parser.diagnostics.emit_to_string());
    if let Some(program) = parsed_program {
        let compiler = Compiler::new();
        let module = compiler.generate_ir(program);
        let mut backend = LLVMBackend::new(&compiler, &module);
        backend.process();
        backend.dump();
    }
}