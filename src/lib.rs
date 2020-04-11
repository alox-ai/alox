extern crate codespan;
extern crate codespan_reporting;
extern crate cranelift_codegen;
extern crate cranelift_faerie;
extern crate cranelift_frontend;
extern crate cranelift_module;
extern crate cuda;
extern crate lalrpop_util;
#[macro_use]
extern crate lazy_static;
extern crate llvm_sys;
extern crate rspirv;
extern crate spirv_headers as spirv;

pub mod diagnostic;
pub mod parser;
pub mod ast;
pub mod ir;
pub mod backend;
pub mod util;
