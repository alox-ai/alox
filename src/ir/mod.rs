pub mod convert;
pub mod types;

use crate::ir::types::Type;
use std::sync::{Arc, Mutex};

#[derive(Clone, Debug)]
pub struct Module {
    pub name: String,
    pub function_headers: Vec<FunctionHeader>,
    pub functions: Vec<Function>,
    // todo: structs, traits,
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
    pub header: FunctionHeader,
    pub blocks: Vec<Arc<Mutex<Block>>>,
}

#[derive(Clone, Debug)]
pub struct Block {
    pub instructions: Vec<Instruction>
}

#[derive(Clone, Debug)]
pub enum Instruction {
    RegisterAssignment(Box<RegisterAssignment>),
    FunctionCall(Box<FunctionCall>),
}

// -- INSTRUCTIONS -- \\

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
