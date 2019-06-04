use std::collections::HashMap;
use std::fmt::{Display, Error, Formatter};
use std::rc::Rc;
use std::sync::{Arc, Mutex, RwLock};

use crate::ast;
use crate::ir::debug::Printer;
use crate::ir::types::Type;

pub mod convert;
pub mod types;
pub mod debug;

// Thread safe reference to a mutable option of a thread safe reference to a declaration
type DeclarationWrapper = Arc<Mutex<Option<Arc<Declaration>>>>;

pub fn wrap_declaration(declaration: Declaration) -> DeclarationWrapper {
    Arc::new(Mutex::new(Some(Arc::new(declaration))))
}

pub struct Compiler {
    pub modules: RwLock<Vec<Module>>,
    pub resolutions_needed: RwLock<Vec<(ast::Path, String, Option<DeclarationKind>, DeclarationWrapper)>>,
}

impl Compiler {
    pub fn new() -> Compiler {
        Compiler {
            modules: RwLock::new(Vec::with_capacity(5)),
            resolutions_needed: RwLock::new(Vec::with_capacity(20)),
        }
    }

    pub fn add_module(&self, module: Module) {
        // update references that are waiting for this module
        let path = module.path.clone().append(module.name.clone());
        let resolutions = self.resolutions_needed.read().unwrap();
        let mut completed_resolutions = Vec::with_capacity(5);

        // go through each needed resolution
        for (i, needed_resolution) in resolutions.iter().enumerate() {
            // this needs the module we are adding
            if needed_resolution.0 == path {
                let mut data = needed_resolution.3.lock().unwrap();
                let declaration = module.resolve(needed_resolution.1.clone(), needed_resolution.2);
                *data = declaration.clone();
                // mark this resolution as completed
                completed_resolutions.push(i);
            }
        }
        drop(resolutions);

        // remove completed resolutions from the list
        let mut writer = self.resolutions_needed.write().unwrap();
        for index in completed_resolutions {
            writer.swap_remove(index);
        }
        self.modules.write().unwrap().push(module);
    }

    pub fn resolve(&self, path: ast::Path, name: String, kind: Option<DeclarationKind>) -> DeclarationWrapper {
        for module in self.modules.read().unwrap().iter() {
            if module.full_path() == path {
                let declaration = module.resolve(name.clone(), kind);
                return Arc::new(Mutex::new(declaration));
            }
        }
        let declaration: DeclarationWrapper = Arc::new(Mutex::new(None));
        let key = (path, name.clone(), kind, declaration.clone());

        self.resolutions_needed.write().unwrap().push(key);
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

    pub fn resolve(&self, name: String, kind: Option<DeclarationKind>) -> Option<Arc<Declaration>> {
        if let Some(kind) = kind {
            for declaration in self.declarations.iter() {
                if declaration.is_declaration_kind(kind) && declaration.name() == name {
                    return Some(declaration.clone());
                }
            }
        } else {
            for declaration in self.declarations.iter() {
                if declaration.name() == name {
                    return Some(declaration.clone());
                }
            }
        }
        None
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialOrd, PartialEq, Hash)]
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

    pub fn is_declaration_kind(&self, kind: DeclarationKind) -> bool {
        let this = self.declaration_kind();
        if this == kind { return true; }
        if kind == DeclarationKind::Type
            && (this == DeclarationKind::Struct
            || this == DeclarationKind::Trait) {
            return true;
        }
        false
    }
}

#[derive(Clone, Debug)]
pub struct Struct {
    pub name: String,
    // Declaration::Variable
    pub fields: Arc<RwLock<Vec<DeclarationWrapper>>>,
    // Declaration::Trait
    pub traits: Arc<RwLock<Vec<DeclarationWrapper>>>,
    // Declaration::FunctionHeader
    pub function_headers: Arc<RwLock<Vec<DeclarationWrapper>>>,
    // Declaration::Function
    pub functions: Arc<RwLock<Vec<DeclarationWrapper>>>,
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

impl Display for Permission {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "+{}{}", self.name, if self.carries { "^" } else { "" })
    }
}

#[derive(Clone, Debug)]
pub struct FunctionHeader {
    pub name: String,
    // assuming declarations are types
    pub arguments: Vec<(String, DeclarationWrapper)>,
    pub return_type: DeclarationWrapper,
    pub refinements: Vec<(String, Vec<Arc<Mutex<Block>>>)>,
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
    pub instructions: Vec<Arc<Mutex<Instruction>>>
}

impl Block {
    pub fn new() -> Block {
        Block {
            instructions: Vec::with_capacity(5)
        }
    }

    pub fn add_instruction(&mut self, instruction: Arc<Mutex<Instruction>>) {
        self.instructions.push(instruction);
    }
}

#[derive(Clone, Debug)]
pub enum Instruction {
    Unreachable(String),
    IntegerLiteral(Box<IntegerLiteral>),
    DeclarationReference(Box<DeclarationReference>),
    FunctionCall(Box<FunctionCall>),
    Return(Box<Return>),
    Branch(Box<Branch>),
}

// -- INSTRUCTIONS -- \\

#[derive(Clone, Debug)]
pub struct IntegerLiteral(pub i64);

#[derive(Clone, Debug)]
pub struct DeclarationReference {
    pub name: (Option<ast::Path>, String),
    pub declaration: DeclarationWrapper,
}

impl DeclarationReference {
    pub fn blank_with_path(path: ast::Path, name: String) -> Self {
        Self {
            name: (Some(path), name),
            declaration: Arc::new(Mutex::new(None)),
        }
    }

    pub fn blank(name: String) -> Self {
        Self {
            name: (None, name),
            declaration: Arc::new(Mutex::new(None)),
        }
    }
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
