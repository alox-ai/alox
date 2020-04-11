#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Path(pub Vec<String>);

impl Path {
    pub fn new() -> Self {
        Path(vec![])
    }

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
    Struct(Box<Struct>),
    Trait(Box<Trait>),
    Function(Box<Function>),
    VariableDeclaration(Box<VariableDeclaration>),
    Error,
}

#[derive(Clone, Debug)]
pub enum Expression {
    BooleanLiteral(Box<BooleanLiteral>),
    IntegerLiteral(Box<IntegerLiteral>),
    VariableReference(Box<VariableReference>),
    FunctionCall(Box<FunctionCall>),
    Error,
}

impl Expression {
    pub fn name(&self) -> String {
        match self {
            Expression::BooleanLiteral(_) => "BooleanLiteral",
            Expression::IntegerLiteral(_) => "IntegerLiteral",
            Expression::VariableReference(_) => "VariableReference",
            Expression::FunctionCall(_) => "FunctionCall",
            Expression::Error => "Error",
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
    Error,
}

// -- NODES -- \\

#[derive(Clone, Debug)]
pub struct Trait {
    pub name: String,
    pub function_declarations: Vec<Function>,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum StructKind {
    Struct,
    Actor,
}

#[derive(Clone, Debug)]
pub struct Struct {
    pub kind: StructKind,
    pub name: String,
    pub traits: Vec<String>,
    pub fields: Vec<VariableDeclaration>,
    pub functions: Vec<Function>,
}

#[derive(Clone, Debug)]
pub struct TypeName {
    pub path: Path,
    pub name: String,
    pub arguments: Vec<Box<TypeName>>,
}

impl TypeName {
    pub fn to_string(&self) -> String {
        let mut name = format!("{}::{}", self.path.to_string(), self.name);
        if self.arguments.len() > 0 {
            name.push_str("[");
            for typ in self.arguments.iter() {
                name.push_str(&typ.to_string());
            }
            name.push_str("]");
        }
        name
    }
}

impl From<(Path, String)> for TypeName {
    fn from(pair: (Path, String)) -> Self {
        Self {
            path: pair.0,
            name: pair.1,
            arguments: vec![],
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum FunctionKind {
    Function,
    Behaviour,
    Kernel,
}

#[derive(Clone, Debug)]
pub struct Function {
    pub kind: FunctionKind,
    pub name: String,
    pub arguments: Vec<(String, TypeName)>,
    pub return_type: TypeName,
    pub statements: Vec<Statement>,
}

#[derive(Clone, Debug)]
pub struct VariableDeclaration {
    pub mutable: bool,
    pub name: String,
    pub type_name: Option<TypeName>,
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
