mod ast;
mod ir;

fn main() {
    let mut program = ast::Program {
        file_name: "bounded.alox".to_string(),
        nodes: vec![],
    };

    // let INT32_MAX: Int32 = 2_147_483_647
    program.nodes.push(ast::Node::VariableDeclaration(Box::new(ast::VariableDeclaration {
        name: "INT32_MAX".to_string(),
        type_name: Some("Int32".to_string()),
        initial_expression: Some(ast::Expression::IntegerLiteral(Box::new(ast::IntegerLiteral(2_147_483_647)))),
    })));

    // fun bounded(n: Int32): Bool
    program.nodes.push(ast::Node::FunctionDeclaration(Box::new(ast::FunctionDeclaration {
        name: "bounded".to_string(),
        arguments: vec![("n".to_string(), "Int32".to_string())],
        return_type: "Bool".to_string(),
        refinements: vec![],
        permissions: vec![],
    })));

    // let bounded = (n) -> {
    //     return (addWithOverflow(n, INT32_MAX) > 0) && (n < INT32_MAX)
    // }
    program.nodes.push(ast::Node::FunctionDefinition(Box::new(ast::FunctionDefinition {
        name: "bounded".to_string(),
        arguments: vec![("n".to_string(), None)],
        statements: vec![
            ast::Statement::Return(Box::new(ast::Return {
                expression: ast::Expression::FunctionCall(Box::new(ast::FunctionCall {
                    function: ast::Expression::VariableReference(Box::new(ast::VariableReference { name: "&&".to_string() })),
                    arguments: vec![
                        ast::Expression::FunctionCall(Box::new(ast::FunctionCall {
                            function: ast::Expression::VariableReference(Box::new(ast::VariableReference { name: ">".to_string() })),
                            arguments: vec![
                                ast::Expression::FunctionCall(Box::new(ast::FunctionCall {
                                    function: ast::Expression::VariableReference(Box::new(ast::VariableReference { name: "addWithOverflow".to_string() })),
                                    arguments: vec![
                                        ast::Expression::VariableReference(Box::new(ast::VariableReference { name: "n".to_string() })),
                                        ast::Expression::VariableReference(Box::new(ast::VariableReference { name: "INT32_MAX".to_string() })),
                                    ],
                                })),
                            ],
                        })),
                        ast::Expression::FunctionCall(Box::new(ast::FunctionCall {
                            function: ast::Expression::VariableReference(Box::new(ast::VariableReference { name: "<".to_string() })),
                            arguments: vec![
                                ast::Expression::VariableReference(Box::new(ast::VariableReference { name: "n".to_string() })),
                                ast::Expression::VariableReference(Box::new(ast::VariableReference { name: "INT32_MAX".to_string() })),
                            ],
                        }))
                    ],
                }))
            }))
        ],
    })));

    // fun add(x: Int32, y: Int32): Int32
    //    where (y: bounded(x + y), return: x + y)
    program.nodes.push(ast::Node::FunctionDeclaration(Box::new(ast::FunctionDeclaration {
        name: "add".to_string(),
        arguments: vec![
            ("x".to_string(), "Int32".to_string()),
            ("y".to_string(), "Int32".to_string())
        ],
        return_type: "Int32".to_string(),
        refinements: vec![
            ("y".to_string(), ast::Expression::FunctionCall(Box::new(ast::FunctionCall {
                function: ast::Expression::VariableReference(Box::new(ast::VariableReference { name: "bounded".to_string() })),
                arguments: vec![
                    ast::Expression::FunctionCall(Box::new(ast::FunctionCall {
                        function: ast::Expression::VariableReference(Box::new(ast::VariableReference { name: "+".to_string() })),
                        arguments: vec![
                            ast::Expression::VariableReference(Box::new(ast::VariableReference { name: "x".to_string() })),
                            ast::Expression::VariableReference(Box::new(ast::VariableReference { name: "y".to_string() })),
                        ],
                    }))
                ],
            }))),
            ("return".to_string(), ast::Expression::FunctionCall(Box::new(ast::FunctionCall {
                function: ast::Expression::VariableReference(Box::new(ast::VariableReference { name: "+".to_string() })),
                arguments: vec![
                    ast::Expression::VariableReference(Box::new(ast::VariableReference { name: "x".to_string() })),
                    ast::Expression::VariableReference(Box::new(ast::VariableReference { name: "y".to_string() })),
                ],
            })))
        ],
        permissions: vec![],
    })));

    // let add = (a, b) -> {
    //     return a + b
    // }
    program.nodes.push(ast::Node::FunctionDefinition(Box::new(ast::FunctionDefinition {
        name: "add".to_string(),
        arguments: vec![
            ("a".to_string(), None),
            ("b".to_string(), None)
        ],
        statements: vec![
            ast::Statement::Return(Box::new(ast::Return {
                expression: ast::Expression::FunctionCall(Box::new(ast::FunctionCall {
                    function: ast::Expression::VariableReference(Box::new(ast::VariableReference { name: "add".to_string() })),
                    arguments: vec![
                        ast::Expression::VariableReference(Box::new(ast::VariableReference { name: "a".to_string() })),
                        ast::Expression::VariableReference(Box::new(ast::VariableReference { name: "b".to_string() })),
                    ],
                }))
            }))
        ],
    })));

    // fun main() +IO
    program.nodes.push(ast::Node::FunctionDeclaration(Box::new(ast::FunctionDeclaration {
        name: "main".to_string(),
        arguments: vec![],
        return_type: "Void".to_string(),
        refinements: vec![],
        permissions: vec!["IO".to_string()],
    })));

    // let main = () -> {
    //     let a = INT32_MAX - 2
    //     let b = 3
    //     // compile time error!
    //     let c = add(a, b)
    //     println(c)
    // }
    program.nodes.push(ast::Node::FunctionDefinition(Box::new(ast::FunctionDefinition {
        name: "main".to_string(),
        arguments: vec![],
        statements: vec![
            ast::Statement::VariableDeclaration(Box::new(ast::VariableDeclaration {
                name: "a".to_string(),
                type_name: None,
                initial_expression: Some(ast::Expression::FunctionCall(Box::new(ast::FunctionCall {
                    function: ast::Expression::VariableReference(Box::new(ast::VariableReference { name: "-".to_string() })),
                    arguments: vec![
                        ast::Expression::VariableReference(Box::new(ast::VariableReference { name: "INT32_MAX".to_string() })),
                        ast::Expression::VariableReference(Box::new(ast::VariableReference { name: "2".to_string() })),
                    ],
                }))),
            })),
            ast::Statement::VariableDeclaration(Box::new(ast::VariableDeclaration {
                name: "b".to_string(),
                type_name: None,
                initial_expression: Some(ast::Expression::IntegerLiteral(Box::new(ast::IntegerLiteral(3)))),
            })),
            ast::Statement::VariableDeclaration(Box::new(ast::VariableDeclaration {
                name: "c".to_string(),
                type_name: None,
                initial_expression: Some(ast::Expression::FunctionCall(Box::new(ast::FunctionCall {
                    function: ast::Expression::VariableReference(Box::new(ast::VariableReference { name: "add".to_string() })),
                    arguments: vec![
                        ast::Expression::VariableReference(Box::new(ast::VariableReference { name: "a".to_string() })),
                        ast::Expression::VariableReference(Box::new(ast::VariableReference { name: "b".to_string() })),
                    ],
                }))),
            })),
            ast::Statement::FunctionCall(Box::new(ast::FunctionCall {
                function: ast::Expression::VariableReference(Box::new(ast::VariableReference { name: "println".to_string() })),
                arguments: vec![
                    ast::Expression::VariableReference(Box::new(ast::VariableReference { name: "c".to_string() }))
                ],
            }))
        ],
    })));

    let module = ir::convert::generate_ir(dbg!(program));
    dbg!(module);
}
