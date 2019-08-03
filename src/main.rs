#[macro_use]
extern crate lazy_static;

use std::sync::Arc;
use std::thread;
use std::time::Instant;

mod ast;
mod ir;

fn main() {
    let mut add_program = ast::Program {
        path: ast::Path::of("test"),
        file_name: "add".to_string(),
        imports: vec![],
        nodes: vec![],
    };

    // let INT32_MAX: Int32 = 2_147_483_647
    add_program
        .nodes
        .push(ast::Node::VariableDeclaration(Box::new(
            ast::VariableDeclaration {
                name: "INT32_MAX".to_string(),
                type_name: Some("Int32".to_string()),
                initial_expression: ast::Expression::IntegerLiteral(Box::new(ast::IntegerLiteral(
                    2_147_483_647,
                ))),
            },
        )));

    // fun bounded(n: Int32): Bool
    add_program
        .nodes
        .push(ast::Node::FunctionDeclaration(Box::new(
            ast::FunctionDeclaration {
                name: "bounded".to_string(),
                arguments: vec![("n".to_string(), (ast::Path(vec![]), "Int32".to_string()))],
                return_type: (ast::Path(vec![]), "Bool".to_string()),
                refinements: vec![],
                permissions: vec![],
            },
        )));

    // let bounded = (n) -> {
    //     return (addWithOverflow(n, INT32_MAX) > 0) && (n < INT32_MAX)
    // }
    add_program
        .nodes
        .push(ast::Node::FunctionDefinition(Box::new(
            ast::FunctionDefinition {
                name: "bounded".to_string(),
                arguments: vec![("n".to_string(), None)],
                statements: vec![ast::Statement::Return(Box::new(ast::Return {
                    expression: ast::Expression::FunctionCall(Box::new(ast::FunctionCall {
                        function: ast::Expression::VariableReference(Box::new(
                            ast::VariableReference::from_str("&&"),
                        )),
                        arguments: vec![
                            ast::Expression::FunctionCall(Box::new(ast::FunctionCall {
                                function: ast::Expression::VariableReference(Box::new(
                                    ast::VariableReference::from_str(">"),
                                )),
                                arguments: vec![ast::Expression::FunctionCall(Box::new(
                                    ast::FunctionCall {
                                        function: ast::Expression::VariableReference(Box::new(
                                            ast::VariableReference::from_str("addWithOverflow"),
                                        )),
                                        arguments: vec![
                                            ast::Expression::VariableReference(Box::new(
                                                ast::VariableReference::from_str("n"),
                                            )),
                                            ast::Expression::VariableReference(Box::new(
                                                ast::VariableReference::from_str("INT32_MAX"),
                                            )),
                                        ],
                                    },
                                ))],
                            })),
                            ast::Expression::FunctionCall(Box::new(ast::FunctionCall {
                                function: ast::Expression::VariableReference(Box::new(
                                    ast::VariableReference::from_str("<"),
                                )),
                                arguments: vec![
                                    ast::Expression::VariableReference(Box::new(
                                        ast::VariableReference::from_str("n"),
                                    )),
                                    ast::Expression::VariableReference(Box::new(
                                        ast::VariableReference::from_str("INT32_MAX"),
                                    )),
                                ],
                            })),
                        ],
                    })),
                }))],
            },
        )));

    // fun add(x: Int32, y: Int32): Int32
    //    where (y: bounded(x + y), return: x + y)
    add_program
        .nodes
        .push(ast::Node::FunctionDeclaration(Box::new(
            ast::FunctionDeclaration {
                name: "add".to_string(),
                arguments: vec![
                    ("x".to_string(), (ast::Path(vec![]), "Int32".to_string())),
                    ("y".to_string(), (ast::Path(vec![]), "Int32".to_string())),
                ],
                return_type: (ast::Path(vec![]), "Int32".to_string()),
                refinements: vec![
                    (
                        "y".to_string(),
                        ast::Expression::FunctionCall(Box::new(ast::FunctionCall {
                            function: ast::Expression::VariableReference(Box::new(
                                ast::VariableReference::from_str("bounded"),
                            )),
                            arguments: vec![ast::Expression::FunctionCall(Box::new(
                                ast::FunctionCall {
                                    function: ast::Expression::VariableReference(Box::new(
                                        ast::VariableReference::from_str("+"),
                                    )),
                                    arguments: vec![
                                        ast::Expression::VariableReference(Box::new(
                                            ast::VariableReference::from_str("x"),
                                        )),
                                        ast::Expression::VariableReference(Box::new(
                                            ast::VariableReference::from_str("y"),
                                        )),
                                    ],
                                },
                            ))],
                        })),
                    ),
                    (
                        "return".to_string(),
                        ast::Expression::FunctionCall(Box::new(ast::FunctionCall {
                            function: ast::Expression::VariableReference(Box::new(
                                ast::VariableReference::from_str("+"),
                            )),
                            arguments: vec![
                                ast::Expression::VariableReference(Box::new(
                                    ast::VariableReference::from_str("x"),
                                )),
                                ast::Expression::VariableReference(Box::new(
                                    ast::VariableReference::from_str("y"),
                                )),
                            ],
                        })),
                    ),
                ],
                permissions: vec![],
            },
        )));

    // let add = (a, b) -> {
    //     return a + b
    // }
    add_program
        .nodes
        .push(ast::Node::FunctionDefinition(Box::new(
            ast::FunctionDefinition {
                name: "add".to_string(),
                arguments: vec![("a".to_string(), None), ("b".to_string(), None)],
                statements: vec![ast::Statement::Return(Box::new(ast::Return {
                    expression: ast::Expression::FunctionCall(Box::new(ast::FunctionCall {
                        function: ast::Expression::VariableReference(Box::new(
                            ast::VariableReference::from_str("+"),
                        )),
                        arguments: vec![
                            ast::Expression::VariableReference(Box::new(
                                ast::VariableReference::from_str("a"),
                            )),
                            ast::Expression::VariableReference(Box::new(
                                ast::VariableReference::from_str("b"),
                            )),
                        ],
                    })),
                }))],
            },
        )));

    let mut main_program = ast::Program {
        path: ast::Path::of("test"),
        file_name: "main".to_string(),
        imports: vec![ast::Path(vec!["test".to_string(), "add".to_string()])],
        nodes: vec![],
    };

    // fun main() +IO
    main_program
        .nodes
        .push(ast::Node::FunctionDeclaration(Box::new(
            ast::FunctionDeclaration {
                name: "main".to_string(),
                arguments: vec![],
                return_type: (ast::Path(vec![]), "Void".to_string()),
                refinements: vec![],
                permissions: vec!["IO".to_string()],
            },
        )));

    // let main = () -> {
    //     let a = INT32_MAX - 2
    //     let b = 3
    //     // compile time error!
    //     let c = add(a, b)
    //     println(c)
    // }
    main_program
        .nodes
        .push(ast::Node::FunctionDefinition(Box::new(
            ast::FunctionDefinition {
                name: "main".to_string(),
                arguments: vec![],
                statements: vec![
                    ast::Statement::VariableDeclaration(Box::new(ast::VariableDeclaration {
                        name: "a".to_string(),
                        type_name: None,
                        initial_expression: ast::Expression::FunctionCall(Box::new(
                            ast::FunctionCall {
                                function: ast::Expression::VariableReference(Box::new(
                                    ast::VariableReference::from_str("-"),
                                )),
                                arguments: vec![
                                    ast::Expression::VariableReference(Box::new(
                                        ast::VariableReference::from_str("INT32_MAX"),
                                    )),
                                    ast::Expression::IntegerLiteral(Box::new(ast::IntegerLiteral(
                                        2,
                                    ))),
                                ],
                            },
                        )),
                    })),
                    ast::Statement::VariableDeclaration(Box::new(ast::VariableDeclaration {
                        name: "b".to_string(),
                        type_name: None,
                        initial_expression: ast::Expression::IntegerLiteral(Box::new(
                            ast::IntegerLiteral(3),
                        )),
                    })),
                    ast::Statement::VariableDeclaration(Box::new(ast::VariableDeclaration {
                        name: "c".to_string(),
                        type_name: None,
                        initial_expression: ast::Expression::FunctionCall(Box::new(
                            ast::FunctionCall {
                                function: ast::Expression::VariableReference(Box::new(
                                    ast::VariableReference {
                                        path: Some(ast::Path(vec![
                                            "test".to_string(),
                                            "add".to_string(),
                                        ])),
                                        name: "add".to_string(),
                                    },
                                )),
                                arguments: vec![
                                    ast::Expression::VariableReference(Box::new(
                                        ast::VariableReference::from_str("a"),
                                    )),
                                    ast::Expression::VariableReference(Box::new(
                                        ast::VariableReference::from_str("b"),
                                    )),
                                ],
                            },
                        )),
                    })),
                    ast::Statement::FunctionCall(Box::new(ast::FunctionCall {
                        function: ast::Expression::VariableReference(Box::new(
                            ast::VariableReference::from_str("println"),
                        )),
                        arguments: vec![ast::Expression::VariableReference(Box::new(
                            ast::VariableReference::from_str("c"),
                        ))],
                    })),
                ],
            },
        )));

    let compiler = Arc::new(ir::Compiler::new());

    // simulate thread pool
    let handle = thread::spawn({
        let compiler_copy = compiler.clone();
        move || {
            //            thread::sleep(std::time::Duration::from_secs(1));
            let now = Instant::now();
            let module = compiler_copy.generate_ir(add_program);
            println!("Add Module: {:?}", now.elapsed());
            compiler_copy.add_module(module);
        }
    });

    let now = Instant::now();
    compiler.add_module(compiler.generate_ir(main_program));
    println!("Main Module: {:?}", now.elapsed());
    handle.join().unwrap();

    let mut printer = ir::debug::Printer::new();
    for module in compiler.modules.read().unwrap().iter() {
        printer.print_module(module);
    }

    let resolutions = compiler.resolutions_needed.read().unwrap();
    for needed_resolution in resolutions.iter() {
        dbg!(needed_resolution);
    }
}
