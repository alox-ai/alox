extern crate alox;

use alox::ast::Path;
use alox::backend::cranelift::CraneLiftBackend;
use alox::ir::Compiler;
use alox::ir::pass::{DeadBranchRemovalPass, Pass};
use alox::parser::Parser;

pub fn check_ir(test_name: &str, code: &str, expected_ir: &str) {
    // parse the module and compiler it to ir
    let mut parser = Parser::new();
    let parsed_program = parser.parse(Path::of("test"), test_name.to_string(), code.to_string());
    let compiler = Compiler::new();

    let mut module = compiler.generate_ir(match parsed_program {
        Some(program) => program,
        None => {
            parser.emit_errors();
            panic!("expected ast");
        }
    });
    let pass = DeadBranchRemovalPass {};
    pass.pass(&mut module);
    compiler.add_module(module);

    // print the module and store it in the buffer
    let mut actual_ir = String::new();
    let backend = CraneLiftBackend::new();
    for module in compiler.modules.read().unwrap().iter() {
        actual_ir = backend.convert_module(&compiler, module);
    }

    // remove trailing new lines
    while actual_ir.ends_with('\n') {
        actual_ir.pop();
    }
    let mut expected_ir = expected_ir.to_string();
    while expected_ir.ends_with('\n') {
        expected_ir.pop();
    }

    println!("========== Expected ==========");
    println!("{}", expected_ir);
    println!("=========== Actual ===========");
    println!("{}", actual_ir);
    println!("==========");
    assert_eq!(actual_ir, expected_ir);
}

#[test]
pub fn return_integer_constant() {
    check_ir("return_integer_constant", "\
fun test(): Int32 {
    return 1
}", "\
function %test() -> i32 system_v {
ebb0:
    v0 = iconst.i64 1
    return v0
}");
}

#[test]
pub fn multiple_functions() {
    check_ir("multiple_functions", "\
fun test(): Int32 {
    return 1
}

fun test2(): Int32 {
    return 2
}", "\
function %test() -> i32 system_v {
ebb0:
    v0 = iconst.i64 1
    return v0
}

function %test2() -> i32 system_v {
ebb0:
    v0 = iconst.i64 2
    return v0
}");
}