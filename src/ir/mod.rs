use std::sync::{Arc, Mutex};

use chashmap::CHashMap;

use crate::ast;
use crate::ir::types::Type;

pub mod convert;
pub mod types;

pub struct Compiler {
    pub modules: Vec<Module>,
    pub resolutions_needed: CHashMap<(ast::Path, String, DeclarationKind), Arc<Mutex<Option<Declaration>>>>,
}

impl Compiler {
    pub fn new() -> Compiler {
        Compiler {
            modules: Vec::with_capacity(5),
            resolutions_needed: CHashMap::new(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Module {
    pub name: String,
    pub declarations: Vec<Arc<Declaration>>,
}

#[derive(Clone, Debug, Eq, Ord, PartialOrd, PartialEq, Hash)]
pub enum DeclarationKind {
    FunctionHeader,
    Function,
}

#[derive(Clone, Debug)]
pub enum Declaration {
    FunctionHeader(Box<FunctionHeader>),
    Function(Box<Function>),
    // todo: structs, traits, variables
}

#[derive(Clone, Debug)]
pub struct Permission {
    pub name: String,
    pub carries: bool,
}

#[derive(Clone, Debug)]
pub struct FunctionHeader {
    pub name: String,
    pub arguments: Vec<(String, Type)>,
    pub return_type: Type,
    pub refinements: Vec<(String, Block)>,
    pub permissions: Vec<Permission>,
}

#[derive(Clone, Debug)]
pub struct Function {
    // assuming this Declaration is a FunctionHeader
    pub header: Arc<Mutex<Option<Arc<Declaration>>>>,
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
}

// -- INSTRUCTIONS -- \\

#[derive(Clone, Debug)]
pub struct DeclarationReference {
    pub name: (ast::Path, String),
    pub declaration: Arc<Mutex<Option<Declaration>>>,
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
