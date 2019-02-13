#[derive(Clone, Debug)]
pub struct Program {
    pub file_name: String,
    pub nodes: Vec<Node>,
}

#[derive(Clone, Debug)]
pub enum Node {
    Struct(Box<Struct>),
    Trait(Box<Trait>),
    FunctionDeclaration(Box<FunctionDeclaration>),
    FunctionDefinition(Box<FunctionDefinition>),
    VariableDeclaration(Box<VariableDeclaration>),
}

#[derive(Clone, Debug)]
pub enum Expression {
    IntegerLiteral(Box<IntegerLiteral>),
    VariableReference(Box<VariableReference>),
    FunctionCall(Box<FunctionCall>),
}

#[derive(Clone, Debug)]
pub enum Statement {
    VariableDeclaration(Box<VariableDeclaration>),
    Return(Box<Return>),
    FunctionCall(Box<FunctionCall>)
}

// -- NODES -- \\

#[derive(Clone, Debug)]
pub struct Trait {
    pub name: String,
    pub function_declarations: Vec<FunctionDeclaration>,
}

#[derive(Clone, Debug)]
pub struct Struct {
    pub name: String,
    pub traits: Vec<String>,
    pub function_declarations: Vec<FunctionDeclaration>,
    pub function_definitions: Vec<FunctionDefinition>,
}

#[derive(Clone, Debug)]
pub struct FunctionDeclaration {
    pub name: String,
    pub arguments: Vec<(String, String)>,
    pub return_type: String,
    pub refinements: Vec<(String, Expression)>,
    pub permissions: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct FunctionDefinition {
    pub name: String,
    pub arguments: Vec<(String, Option<String>)>,
    pub statements: Vec<Statement>,
}

#[derive(Clone, Debug)]
pub struct VariableDeclaration {
    pub name: String,
    pub type_name: Option<String>,
    pub initial_expression: Option<Expression>,
}

// -- STATEMENTS -- \\

#[derive(Clone, Debug)]
pub struct Return { pub expression: Expression }

// -- EXPRESSIONS -- \\

#[derive(Clone, Debug)]
pub struct IntegerLiteral(pub i64);

#[derive(Clone, Debug)]
pub struct VariableReference {
    pub name: String,
}

#[derive(Clone, Debug)]
pub struct FunctionCall {
    pub function: Expression,
    pub arguments: Vec<Expression>,
}
