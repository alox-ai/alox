#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Path(pub Vec<String>);

impl Path {
    pub fn of(s: &str) -> Self {
        Self(vec![s.to_string()])
    }
    pub fn append(&self, s: String) -> Self {
        let mut vec = self.0.clone();
        vec.push(s);
        Self(vec)
    }
}

#[derive(Clone, Debug)]
pub struct Program {
    pub path: Path,
    pub file_name: String,
    pub imports: Vec<Path>,
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
    FunctionCall(Box<FunctionCall>),
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
    pub arguments: Vec<(String, (Path, String))>,
    pub return_type: (Path, String),
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

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct VariableReference {
    pub path: Option<Path>,
    pub name: String,
}

impl VariableReference {
    pub fn from_str(name: &str) -> VariableReference {
        VariableReference {
            path: None,
            name: name.to_string(),
        }
    }

    pub fn from_str_with_path(path: Path, name: &str) -> VariableReference {
        VariableReference {
            path: Some(path),
            name: name.to_string(),
        }
    }

    pub fn from_string(name: String) -> VariableReference {
        VariableReference {
            path: None,
            name,
        }
    }
}

#[derive(Clone, Debug)]
pub struct FunctionCall {
    pub function: Expression,
    pub arguments: Vec<Expression>,
}
