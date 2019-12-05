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

    pub fn to_string(&self) -> String {
        self.0.join("::")
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
    Actor(Box<Actor>),
    Struct(Box<Struct>),
    Trait(Box<Trait>),
    Function(Box<Function>),
    VariableDeclaration(Box<VariableDeclaration>),
}

#[derive(Clone, Debug)]
pub enum Expression {
    BooleanLiteral(Box<BooleanLiteral>),
    IntegerLiteral(Box<IntegerLiteral>),
    VariableReference(Box<VariableReference>),
    FunctionCall(Box<FunctionCall>),
}

impl Expression {
    pub fn name(&self) -> String {
        match self {
            Expression::BooleanLiteral(_) => "BooleanLiteral",
            Expression::IntegerLiteral(_) => "IntegerLiteral",
            Expression::VariableReference(_) => "VariableReference",
            Expression::FunctionCall(_) => "FunctionCall",
        }
        .to_string()
    }
}

#[derive(Clone, Debug)]
pub enum Statement {
    VariableDeclaration(Box<VariableDeclaration>),
    If(Box<IfStatement>),
    Return(Box<Return>),
    FunctionCall(Box<FunctionCall>),
}

// -- NODES -- \\

#[derive(Clone, Debug)]
pub struct Trait {
    pub name: String,
    pub function_declarations: Vec<Function>,
}

#[derive(Clone, Debug)]
pub struct Struct {
    pub name: String,
    pub traits: Vec<String>,
    pub fields: Vec<VariableDeclaration>,
    pub functions: Vec<Function>,
}

#[derive(Clone, Debug)]
pub struct Actor {
    pub name: String,
    pub fields: Vec<VariableDeclaration>,
    pub functions: Vec<Function>,
    pub behaviours: Vec<Behaviour>,
}

#[derive(Clone, Debug)]
pub struct Function {
    pub name: String,
    pub arguments: Vec<(String, (Path, String))>,
    pub return_type: (Path, String),
    pub statements: Vec<Statement>,
}

#[derive(Clone, Debug)]
pub struct Behaviour {
    pub name: String,
    pub arguments: Vec<(String, (Path, String))>,
    pub statements: Vec<Statement>,
}

#[derive(Clone, Debug)]
pub struct VariableDeclaration {
    pub name: String,
    pub type_name: Option<(Path, String)>,
    pub initial_expression: Option<Expression>,
}

// -- STATEMENTS -- \\

#[derive(Clone, Debug)]
pub struct Return {
    pub expression: Expression,
}

#[derive(Clone, Debug)]
pub struct IfStatement {
    pub condition: Expression,
    pub block: Vec<Statement>,
    pub elseif: Option<Box<IfStatement>>,
}

// -- EXPRESSIONS -- \\

#[derive(Clone, Debug)]
pub struct BooleanLiteral(pub bool);

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
        VariableReference { path: None, name }
    }
}

#[derive(Clone, Debug)]
pub struct FunctionCall {
    pub function: Expression,
    pub arguments: Vec<Expression>,
}
