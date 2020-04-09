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
extern crate rspirv;
extern crate spirv_headers as spirv;

use crate::backend::cranelift::CraneLiftBackend;
use crate::ir::debug::PrintMode;
use crate::ir::pass::{DeadBranchRemovalPass, Pass};

pub mod diagnostic;
pub mod parser;
pub mod ast;
pub mod ir;
pub mod backend;
pub mod util;

fn main() {
    let test_file = "test.alox";
    let test_source = "\
    fun main(x: a::aa::A, fun: b::bb::B, z: c::cc::C): d::dd::D {
        let a = 3
        let b = test(a, 2)
        return b
    }".to_string();
    let mut parser = parser::Parser::new();
    let parsed_program = parser.parse(ast::Path::of("test"), test_file.to_string(), test_source);
    println!("{:#?}", parsed_program);
    parser.emit_errors();
}