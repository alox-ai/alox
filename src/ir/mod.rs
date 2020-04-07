use std::fmt::{Display, Error, Formatter};
use std::sync::RwLock;

use crate::ast;
use crate::ir::types::{PrimitiveType, Type};
use crate::util::Either;
use std::collections::HashMap;

pub mod convert;
pub mod debug;
pub mod types;
pub mod builtin;
pub mod pass;

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct DeclarationId(pub String);

impl DeclarationId {
    pub fn from(module: Option<&Module>, declaration: &Declaration) -> Self {
        let declaration_id = declaration.name();
        if let Some(module) = module {
            let module_id = module.full_path().to_string();
            DeclarationId(format!("{}::{}", module_id, declaration_id))
        } else {
            DeclarationId(declaration_id)
        }
    }

    pub fn get_type(&self) -> Box<types::Type> {
        Box::new(types::Type::Unresolved(types::UnresolvedType { name: self.0.clone() }))
    }

    pub fn name(&self) -> String {
        self.0.split("::").last().unwrap().to_string()
    }
}

pub struct Compiler<'compiler> {
    pub modules: RwLock<Vec<Module<'compiler>>>,
    pub declaration_bank: RwLock<HashMap<DeclarationId, usize>>,
}

impl<'compiler> Compiler<'compiler> {
    pub fn new() -> Compiler<'compiler> {
        Compiler {
            modules: RwLock::new(Vec::with_capacity(5)),
            declaration_bank: RwLock::new(HashMap::new()),
        }
    }

    pub fn add_module(&'compiler self, module: Module<'compiler>) {
        let mut bank = self.declaration_bank.write().unwrap();
        for declaration in module.declarations.iter() {
            let declaration_id = DeclarationId::from(Some(&module), declaration);
            if bank.contains_key(&declaration_id) {
                panic!("ahh oh no the declaration already exists!!!");
            }
            let dec_pointer = declaration as *const Declaration as usize;
            let _ = bank.insert(declaration_id, dec_pointer);
        }
        drop(bank);

        self.modules.write().unwrap().push(module);
    }

    pub fn resolve_from_path(
        &self,
        path: ast::Path,
        name: String,
    ) -> Option<&'compiler Declaration> {
        let declaration_id = DeclarationId(format!("{}::{}", path.to_string(), name));
        self.resolve(&declaration_id)
    }

    pub fn resolve(&self, declaration_id: &DeclarationId) -> Option<&'compiler Declaration> {
        if let Some(declaration) = self.declaration_bank.read().unwrap().get(declaration_id) {
            let dec_ptr = *declaration as *const Declaration<'compiler>;
            Some(unsafe { &*dec_ptr })
        } else { None }
    }
}

#[derive(Clone, Debug)]
pub struct Module<'a> {
    /// path doesn't contain the module's name
    pub path: ast::Path,
    pub name: String,
    pub declarations: Vec<Declaration<'a>>,
}

impl Module<'_> {
    pub fn full_path(&self) -> ast::Path {
        self.path.append(self.name.clone())
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialOrd, PartialEq, Hash)]
pub enum DeclarationKind {
    Behaviour,
    Function,
    Actor,
    Struct,
    Trait,
    Variable,
    Type,
}

#[derive(Clone, Debug)]
pub enum Declaration<'dec> {
    Behaviour(Box<Behaviour<'dec>>),
    Function(Box<Function<'dec>>),
    Actor(Box<Actor<'dec>>),
    Struct(Box<Struct<'dec>>),
    Trait(Box<Trait<'dec>>),
    Variable(Box<Variable>),
    Type(Box<Type>),
}

impl<'a> Declaration<'a> {
    pub fn name(&self) -> String {
        match self {
            Declaration::Behaviour(b) => b.name.clone(),
            Declaration::Function(f) => f.name.clone(),
            Declaration::Actor(a) => a.name.clone(),
            Declaration::Struct(s) => s.name.clone(),
            Declaration::Trait(t) => t.name.clone(),
            Declaration::Variable(v) => v.name.clone(),
            Declaration::Type(t) => t.name().clone(),
        }
    }

    pub fn declaration_kind(&self) -> DeclarationKind {
        match self {
            Declaration::Behaviour(_) => DeclarationKind::Behaviour,
            Declaration::Function(_) => DeclarationKind::Function,
            Declaration::Actor(_) => DeclarationKind::Actor,
            Declaration::Struct(_) => DeclarationKind::Struct,
            Declaration::Trait(_) => DeclarationKind::Trait,
            Declaration::Variable(_) => DeclarationKind::Variable,
            Declaration::Type(_) => DeclarationKind::Type,
        }
    }

    pub fn is_declaration_kind(&self, kind: DeclarationKind) -> bool {
        let this = self.declaration_kind();
        if this == kind {
            return true;
        }
        if kind == DeclarationKind::Type
            && (this == DeclarationKind::Struct
            || this == DeclarationKind::Trait
            || this == DeclarationKind::Function
            || this == DeclarationKind::Behaviour
            || this == DeclarationKind::Actor)
        {
            return true;
        }
        false
    }

    pub fn get_type(&self) -> Box<Type> {
        match self {
            Declaration::Function(f) => f.get_type(),
            Declaration::Struct(s) => s.get_type(),
            Declaration::Type(t) => t.clone(),
            _ => Box::new(types::Type::Unresolved(types::UnresolvedType::of("UnresolvedDeclaration"))),
        }
    }

    pub fn is_type(&self) -> bool {
        let kind = self.declaration_kind();
        kind == DeclarationKind::Type
            || kind == DeclarationKind::Struct
            || kind == DeclarationKind::Trait
            || kind == DeclarationKind::Function
    }

    pub fn is_same_type(&self, declaration: &Declaration) -> bool {
        // one of these isn't a type
        if !self.is_type() || !declaration.is_type() {
            return false;
        }
        // we're comparing different kinds of types
        if self.declaration_kind() != declaration.declaration_kind() {
            return false;
        }
        // compare the pointers
        self as *const _ == declaration as *const _
    }
}

#[derive(Clone, Debug)]
pub struct Actor<'actor> {
    pub name: String,
    // Declaration::Variable
    pub fields: Vec<Declaration<'actor>>,
    // Declaration::Function
    pub functions: Vec<Declaration<'actor>>,
    // Declaration::Behaviour
    pub behaviours: Vec<Declaration<'actor>>,
}

#[derive(Clone, Debug)]
pub struct Struct<'strct> {
    pub name: String,
    // Declaration::Variable
    pub fields: Vec<Declaration<'strct>>,
    // Declaration::Trait
    pub traits: Vec<Declaration<'strct>>,
    // Declaration::Function
    pub functions: Vec<Declaration<'strct>>,
}

impl<'strct> Struct<'strct> {
    pub fn get_type(&self) -> Box<types::Type> {
        let mut fields = Vec::with_capacity(self.fields.len());
        for field in self.fields.iter() {
            let name = field.name();
            let typ = field.get_type();
            fields.push((name, typ));
        }
        Box::new(types::Type::Struct(types::StructType { name: self.name.clone(), fields }))
    }
}

#[derive(Clone, Debug)]
pub struct Trait<'a> {
    pub name: String,
    pub functions: Vec<Function<'a>>,
}

#[derive(Clone, Debug)]
pub struct Variable {
    pub name: String,
    pub typ: DeclarationId,
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
pub struct Function<'a> {
    pub name: String,
    pub arguments: Vec<(String, DeclarationId)>,
    pub return_type: DeclarationId,
    pub blocks: Vec<Block<'a>>,
}

impl<'a> Function<'a> {
    pub fn get_type(&self) -> Box<types::Type> {
        let mut arguments = Vec::<Box<types::Type>>::with_capacity(self.arguments.len());

        for arg in &self.arguments {
            arguments.push(arg.1.get_type());
        }

        let result = self.return_type.get_type();
        Box::new(types::Type::Function(types::FunctionType { arguments, result }))
    }
}

#[derive(Clone, Debug)]
pub struct Behaviour<'a> {
    pub name: String,
    pub arguments: Vec<(String, DeclarationId)>,
    pub blocks: Vec<Block<'a>>,
}

#[derive(Clone, Debug)]
pub struct Block<'block> {
    pub instructions: Vec<Instruction<'block>>,
}

impl<'block> Block<'block> {
    pub fn new() -> Block<'block> {
        Block {
            instructions: Vec::with_capacity(5),
        }
    }

    pub fn add_instruction<'compiler>(&'block mut self, instruction: Instruction<'block>, compiler: &'compiler Compiler<'compiler>) -> &'block Instruction<'block> where 'block: 'compiler {
        // don't add the instruction to this block if it already has an instruction
        // that doesn't return, like Return, Branch, Jump, etc
        {
            let mut found = false;
            let mut found_ins = 0;
            for (index, instruction) in self.instructions.iter().enumerate() {
                match *instruction.get_type(compiler) {
                    Type::Primitive(PrimitiveType::NoReturn) => {
                        found_ins = index;
                        found = true;
                        break;
                    }
                    _ => {}
                }
            }
            if found {
                return self.instructions.get(found_ins).expect("couldn't find instruction we just found?");
            }
        }
        self.instructions.push(instruction);
        self.instructions.get(self.instructions.len() - 1).expect("couldn't find instruction we just added?")
    }
}

#[derive(Clone, Debug)]
pub enum Instruction<'a> {
    Unreachable(String),
    BooleanLiteral(Box<BooleanLiteral>),
    IntegerLiteral(Box<IntegerLiteral>),
    DeclarationReference(Box<DeclarationReference>),
    GetParameter(Box<GetParameter>),
    FunctionCall(Box<FunctionCall<'a>>),
    Return(Box<Return<'a>>),
    Jump(Box<Jump<'a>>),
    Branch(Box<Branch<'a>>),
}

impl<'ins> Instruction<'ins> {
    pub fn get_type<'compiler: 'ins>(&self, compiler: &'compiler Compiler<'compiler>) -> Box<Type> {
        return match self {
            Instruction::BooleanLiteral(_) => builtin::BOOL.get_type(),
            Instruction::IntegerLiteral(_) => builtin::COMPTIME_INT.get_type(),
            Instruction::DeclarationReference(s) => s.declaration.get_type(),
            Instruction::GetParameter(_) => Box::new(types::Type::Unresolved(types::UnresolvedType { name: "UnimplementedParamGet".to_string() })),
            Instruction::FunctionCall(f) => {
                match *f.function {
                    Instruction::DeclarationReference(ref f) => {
                        if let Some(ref dec) = compiler.resolve(&f.declaration) {
                            match *dec {
                                Declaration::Function(ref h) => h.return_type.get_type(),
                                _ => Box::new(types::Type::Unresolved(types::UnresolvedType { name: "UnPointerToFuncBody".to_string() }))
                            }
                        } else {
                            Box::new(types::Type::Unresolved(types::UnresolvedType { name: "UnNoFunctionDec".to_string() }))
                        }
                    }
                    _ => Box::new(types::Type::Unresolved(types::UnresolvedType { name: "UnDecNotFunc".to_string() }))
                }
            }
            Instruction::Return(_)
            | Instruction::Unreachable(_)
            | Instruction::Jump(_)
            | Instruction::Branch(_) => Box::new(types::Type::Primitive(types::PrimitiveType::NoReturn)),
            _ => Box::new(types::Type::Unresolved(types::UnresolvedType { name: "UnknownInstruction".to_string() }))
        };
    }

    /// Get type of an instruction in the context of a function or behaviour.
    /// Useful for getting the type of parameters.
    pub fn get_type_with_context<'compiler: 'ins>(&'ins self, compiler: &'compiler Compiler<'compiler>, context: Either<&Box<Function>, &Box<Behaviour>>) -> Box<Type> {
        match self {
            Instruction::GetParameter(g) => {
                let name = &g.name;
                match context {
                    Either::Left(f) => {
                        for (arg_name, declaration) in &f.arguments {
                            if arg_name == name {
                                return if let Some(declaration) = compiler.resolve(declaration) {
                                    declaration.get_type()
                                } else {
                                    declaration.get_type()
                                };
                            }
                        }
                    }
                    Either::Right(b) => {
                        for (arg_name, declaration) in &b.arguments {
                            if arg_name == name {
                                return if let Some(declaration) = compiler.resolve(declaration) {
                                    declaration.get_type()
                                } else {
                                    declaration.get_type()
                                };
                            }
                        }
                    }
                }
            }
            _ => {}
        }
        self.get_type(compiler)
    }
}

// -- INSTRUCTIONS -- \\

#[derive(Clone, Debug)]
pub struct BooleanLiteral(pub bool);

#[derive(Clone, Debug)]
pub struct IntegerLiteral(pub i64);

#[derive(Clone, Debug)]
pub struct DeclarationReference {
    pub name: (Option<ast::Path>, String),
    pub declaration: DeclarationId,
}

#[derive(Clone, Debug)]
pub struct GetParameter {
    pub name: String,
}

#[derive(Clone, Debug)]
pub struct FunctionCall<'block> {
    pub function: &'block Instruction<'block>,
    pub arguments: Vec<&'block Instruction<'block>>,
}

#[derive(Clone, Debug)]
pub struct Return<'block> {
    pub instruction: &'block Instruction<'block>,
}

#[derive(Clone, Debug)]
pub struct Jump<'block> {
    pub block: &'block Block<'block>,
}

#[derive(Clone, Debug)]
pub struct Branch<'block> {
    pub condition: &'block Instruction<'block>,
    pub true_block: &'block Block<'block>,
    pub false_block: &'block Block<'block>,
}
