#[macro_use]
extern crate lazy_static;
extern crate logos;
extern crate cranelift_codegen;
extern crate cranelift_frontend;
extern crate cuda;
extern crate rspirv;
extern crate spirv_headers as spirv;

pub mod parser;
pub mod ast;
pub mod ir;
pub mod backend;
pub mod util;
