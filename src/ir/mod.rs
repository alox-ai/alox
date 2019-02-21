use std::sync::{Arc, Mutex};

use chashmap::CHashMap;

use crate::ast;
use crate::ir::types::Type;

pub mod convert;
pub mod types;

// Thread safe reference to a mutable option of a thread safe reference to a declaration
type DeclarationWrapper = Arc<Mutex<Option<Arc<Declaration>>>>;

pub struct Compiler {
    pub modules: Vec<Module>,
    pub resolutions_needed: CHashMap<(ast::Path, String, DeclarationKind), DeclarationWrapper>,
}

impl Compiler {
    pub fn new() -> Compiler {
        Compiler {
            modules: Vec::with_capacity(5),
            resolutions_needed: CHashMap::new(),
        }
    }

    pub fn resolve(&self, path: ast::Path, name: String, kind: DeclarationKind) -> DeclarationWrapper {
        for module in self.modules.iter() {
            if module.full_path() == path {
                println!("found module {:?}", path);
                let declaration = module.resolve(name, kind);
                return Arc::new(Mutex::new(declaration));
            }
        }
        let declaration: DeclarationWrapper = Arc::new(Mutex::new(None));
        self.resolutions_needed.insert((path, name, kind), declaration.clone());
        declaration
    }
}

#[derive(Clone, Debug)]
pub struct Module {
    /// path doesn't contain the module's name
    pub path: ast::Path,
    pub name: String,
    pub declarations: Vec<Arc<Declaration>>,
}

impl Module {
    pub fn full_path(&self) -> ast::Path {
        self.path.append(self.name.clone())
    }

    pub fn resolve(&self, name: String, kind: DeclarationKind) -> Option<Arc<Declaration>> {
        for declaration in self.declarations.iter() {
            if declaration.declaration_kind() == kind && declaration.name() == name {
                return Some(declaration.clone());
            }
        }
        None
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialOrd, PartialEq, Hash)]
pub enum DeclarationKind {
    FunctionHeader,
    Function,
    Struct,
    Trait,
    Variable,
    Type,
}

#[derive(Clone, Debug)]
pub enum Declaration {
    FunctionHeader(Box<FunctionHeader>),
    Function(Box<Function>),
    Struct(Box<Struct>),
    Trait(Box<Trait>),
    Variable(Box<Variable>),
    Type(Box<Type>),
}

impl Declaration {
    pub fn name(&self) -> String {
        match self {
            Declaration::FunctionHeader(f) => f.name.clone(),
            Declaration::Function(f) => f.name.clone(),
            Declaration::Struct(s) => s.name.clone(),
            Declaration::Trait(t) => t.name.clone(),
            Declaration::Variable(v) => v.name.clone(),
            Declaration::Type(t) => t.name().clone(),
        }
    }

    pub fn declaration_kind(&self) -> DeclarationKind {
        match self {
            Declaration::FunctionHeader(_) => DeclarationKind::FunctionHeader,
            Declaration::Function(_) => DeclarationKind::Function,
            Declaration::Struct(_) => DeclarationKind::Struct,
            Declaration::Trait(_) => DeclarationKind::Trait,
            Declaration::Variable(_) => DeclarationKind::Variable,
            Declaration::Type(_) => DeclarationKind::Type,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Struct {
    pub name: String,
    pub traits: Arc<Mutex<Vec<Trait>>>,
    pub functions: Vec<Function>,
}

#[derive(Clone, Debug)]
pub struct Trait {
    pub name: String,
    pub function_headers: Vec<FunctionHeader>,
}

#[derive(Clone, Debug)]
pub struct Variable {
    pub name: String,
}

#[derive(Clone, Debug)]
pub struct Permission {
    pub name: String,
    pub carries: bool,
}

#[derive(Clone, Debug)]
pub struct FunctionHeader {
    pub name: String,
    // assuming declarations are types
    pub arguments: Vec<(String, DeclarationWrapper)>,
    pub return_type: DeclarationWrapper,
    pub refinements: Vec<(String, Block)>,
    pub permissions: Vec<Permission>,
}

#[derive(Clone, Debug)]
pub struct Function {
    pub name: String,
    // assuming this Declaration is a FunctionHeader
    pub header: DeclarationWrapper,
    pub blocks: Vec<Arc<Mutex<Block>>>,
}

#[derive(Clone, Debug)]
pub struct Block {
    pub instructions: Vec<Instruction>
}

#[derive(Clone, Debug)]
pub enum Instruction {
    DeclarationReference(Box<DeclarationReference>),
    RegisterAssignment(Box<RegisterAssignment>),
    FunctionCall(Box<FunctionCall>),
    Return(Box<Return>),
    Branch(Box<Branch>),
}

// -- INSTRUCTIONS -- \\

#[derive(Clone, Debug)]
pub struct DeclarationReference {
    pub name: (ast::Path, String),
    pub declaration: DeclarationWrapper,
}

#[derive(Clone, Debug)]
pub struct RegisterAssignment {
    pub name: String,
    pub instruction: Arc<Mutex<Instruction>>,
}

#[derive(Clone, Debug)]
pub struct FunctionCall {
    pub function: Arc<Mutex<Instruction>>,
    pub arguments: Vec<Arc<Mutex<Instruction>>>,
}

#[derive(Clone, Debug)]
pub struct Return {
    pub instruction: Arc<Mutex<Instruction>>,
}

#[derive(Clone, Debug)]
pub struct Branch {
    pub block: Arc<Mutex<Block>>,
}
