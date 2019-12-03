extern crate alox;

use alox::ast::Path;
use alox::ir::debug::{Printer, PrintMode};
use alox::parser;
use alox::ir::Compiler;

pub fn check_ir(test_name: &str, module: &str, expected_ir: &str) {
    // parse the module and compiler it to ir
    let mut parsed_program = parser::parse(Path::of("test"), test_name.to_string(), module.to_string());
    let compiler = Compiler::new();
    compiler.add_module(compiler.generate_ir(parsed_program.unwrap()));

    // print the module and store it in the buffer
    let mut printer = Printer::new(PrintMode::Buffer);
    for module in compiler.modules.read().unwrap().iter() {
        printer.print_module(module);
    }

    // remove trailing new lines
    if printer.buffer.ends_with('\n') {
        printer.buffer.pop();
    }
    let mut expected_ir = expected_ir.to_string();
    if expected_ir.ends_with('\n') {
        expected_ir.pop();
    }

    println!("========== Expected ==========");
    println!("{}", expected_ir);
    println!("=========== Actual ===========");
    println!("{}", printer.buffer);
    println!("==========");
    assert_eq!(printer.buffer, expected_ir);
}

#[test]
pub fn basic_function() {
    check_ir("basic_function", "\
fun test(a: Int32): Int32 {
    return a
}", "\
; Module: test::basic_function
fun @test(%a: Int32) -> Int32:
  block#0:
    %0 : Int32 = param %a
    ret %0");
}

#[test]
pub fn function_call() {
    check_ir("function_call", "\
fun test(a: Int32): Int32 {
    return a
}

fun bar(a: Int32): Int32 {
    return test(a)
}", "\
; Module: test::function_call
fun @test(%a: Int32) -> Int32:
  block#0:
    %0 : Int32 = param %a
    ret %0
fun @bar(%a: Int32) -> Int32:
  block#0:
    %0 : Int32 -> Int32 = @test::function_call::test
    %1 : Int32 = param %a
    %2 : Int32 = %0(%1)
    ret %2");
}

#[test]
pub fn nested_expression() {
    check_ir("nested_expression", "\
fun foo(c: Int32): Int32 {
    return c
}

fun bar(g: Int32): Int32 {
    return foo(foo(foo(foo(foo(g)))))
}
", "\
; Module: test::nested_expression
fun @foo(%c: Int32) -> Int32:
  block#0:
    %0 : Int32 = param %c
    ret %0
fun @bar(%g: Int32) -> Int32:
  block#0:
    %0 : Int32 -> Int32 = @test::nested_expression::foo
    %1 : Int32 -> Int32 = @test::nested_expression::foo
    %2 : Int32 -> Int32 = @test::nested_expression::foo
    %3 : Int32 -> Int32 = @test::nested_expression::foo
    %4 : Int32 -> Int32 = @test::nested_expression::foo
    %5 : Int32 = param %g
    %6 : Int32 = %4(%5)
    %7 : Int32 = %3(%6)
    %8 : Int32 = %2(%7)
    %9 : Int32 = %1(%8)
    %10 : Int32 = %0(%9)
    ret %10
")
}

#[test]
pub fn fields_in_struct() {
    check_ir("fields_in_struct", "\
struct X {
    let x: Int32
    let y: Float32
    let b: Bool
}
", "\
; Module: test::fields_in_struct
struct X:
  let x: Int32
  let y: Float32
  let b: Bool
")
}

#[test]
pub fn fields_in_actor() {
    check_ir("fields_in_actor", "\
actor A {
    let x: Int32
    let y: Float32
    let b: Bool
}
", "\
; Module: test::fields_in_actor
actor A:
  let x: Int32
  let y: Float32
  let b: Bool
")
}

#[test]
pub fn methods_in_struct() {
    check_ir("method_in_struct", "\
struct X {
    let x: Int32
    let y: Float32
    let b: Bool

    fun fooX(a: Int32): Int32 {
        return a
    }
}
", "\
; Module: test::method_in_struct
struct X:
  let x: Int32
  let y: Float32
  let b: Bool
  fun @fooX(%a: Int32) -> Int32:
    block#0:
      %0 : Int32 = param %a
      ret %0
")
}

#[test]
pub fn method_in_actor() {
    check_ir("method_in_actor", "\
actor A {
    let x: Int32
    let y: Float32
    let b: Bool

    fun fooA(a: Int32): Int32 {
        return a
    }
}
", "\
; Module: test::method_in_actor
actor A:
  let x: Int32
  let y: Float32
  let b: Bool
  fun @fooA(%a: Int32) -> Int32:
    block#0:
      %0 : Int32 = param %a
      ret %0
")
}

#[test]
pub fn void_function() {
    check_ir("void_function", "\
fun test(a: Int32) {
}", "\
; Module: test::void_function
fun @test(%a: Int32) -> Void:
");
}
