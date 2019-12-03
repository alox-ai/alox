use std::fmt::{Display, Error, Formatter};
use std::sync::{Arc, Mutex, RwLock};

use crate::ast;
use crate::ir::types::{PrimitiveType, Type};
use crate::util::Either;

pub mod convert;
pub mod debug;
pub mod types;
pub mod builtin;

// Thread safe reference to a mutable option of a thread safe reference to a declaration
// acts as a declaration "hole" that needs to be filled
#[derive(Clone, Debug)]
pub struct DeclarationContainer(pub Arc<Mutex<Option<Arc<Declaration>>>>);

impl DeclarationContainer {
    pub fn from(declaration: Declaration) -> Self {
        Self(Arc::new(Mutex::new(Some(Arc::new(declaration)))))
    }

    pub fn empty() -> Self {
        Self(Arc::new(Mutex::new(None)))
    }

    pub fn is_same_type(&self, other: &Self) -> bool {
        let self_guard = self.0.lock().unwrap();
        if let Some(ref self_dec) = *self_guard {
            let other_guard = other.0.lock().unwrap();
            if let Some(ref other_dec) = *other_guard {
                return self_dec.is_same_type(other_dec);
            }
        }
        false
    }

    pub fn name(&self) -> String {
        let guard = self.0.lock().unwrap();
        if let Some(ref dec) = *guard {
            return dec.name();
        }
        String::from("notfound")
    }

    pub fn get_type(&self) -> Box<Type> {
        let guard = self.0.lock().unwrap();
        if let Some(ref dec) = *guard {
            return dec.get_type();
        }
        Box::new(types::Type::Unresolved(types::UnresolvedType { name: "UnresolvedReference".to_string() }))
    }
}

pub struct Compiler {
    pub modules: RwLock<Vec<Module>>,
    pub resolutions_needed: RwLock<
        Vec<(
            ast::Path,
            String,
            Option<DeclarationKind>,
            DeclarationContainer,
        )>,
    >,
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
                let mut data = (needed_resolution.3).0.lock().unwrap();
                let declaration = module.resolve(needed_resolution.1.clone(), needed_resolution.2);
                // we want to replace the data only if we have some
                if let Some(_) = declaration {
                    *data = declaration;
                    // mark this resolution as completed
                    completed_resolutions.push(i);
                }
            }
        }
        drop(resolutions);

        // remove completed resolutions from the list
        let mut writer = self.resolutions_needed.write().unwrap();
        let mut diff = 0;
        for i in completed_resolutions {
            writer.swap_remove(i - diff);
            diff += 1;
        }
        drop(writer);
        self.modules.write().unwrap().push(module);
    }

    pub fn resolve(
        &self,
        path: ast::Path,
        name: String,
        kind: Option<DeclarationKind>,
    ) -> DeclarationContainer {
        if path.0.len() == 0 {
            if let Some(builtin) = builtin::find_builtin_declaration(name.clone(), kind) {
                return builtin;
            }
        }

        for module in self.modules.read().unwrap().iter() {
            if module.full_path() == path {
                let declaration = module.resolve(name.clone(), kind);
                return DeclarationContainer(Arc::new(Mutex::new(declaration)));
            }
        }
        if let Some(primitive) = PrimitiveType::from_name(name.clone()) {
            return DeclarationContainer::from(Declaration::Type(
                Box::new(Type::Primitive(primitive))
            ));
        }

        let declaration = DeclarationContainer::empty();
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
    Behaviour,
    Function,
    Actor,
    Struct,
    Trait,
    Variable,
    Type,
}

#[derive(Clone, Debug)]
pub enum Declaration {
    Behaviour(Box<Behaviour>),
    Function(Box<Function>),
    Actor(Box<Actor>),
    Struct(Box<Struct>),
    Trait(Box<Trait>),
    Variable(Box<Variable>),
    Type(Box<Type>),
}

impl Declaration {
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
pub struct Actor {
    pub name: String,
    // Declaration::Variable
    pub fields: Arc<RwLock<Vec<DeclarationContainer>>>,
    // Declaration::Function
    pub functions: Arc<RwLock<Vec<DeclarationContainer>>>,
    // Declaration::Behaviour
    pub behaviours: Arc<RwLock<Vec<DeclarationContainer>>>,
}

#[derive(Clone, Debug)]
pub struct Struct {
    pub name: String,
    // Declaration::Variable
    pub fields: Arc<RwLock<Vec<DeclarationContainer>>>,
    // Declaration::Trait
    pub traits: Arc<RwLock<Vec<DeclarationContainer>>>,
    // Declaration::Function
    pub functions: Arc<RwLock<Vec<DeclarationContainer>>>,
}

impl Struct {
    pub fn get_type(&self) -> Box<types::Type> {
        let guard = self.fields.read().unwrap();
        let mut fields = Vec::with_capacity(guard.len());
        for field in guard.iter() {
            let name = field.name();
            let typ = field.get_type();
            fields.push((name, typ));
        }
        Box::new(types::Type::Struct(types::StructType { name: self.name.clone(), fields }))
    }
}

#[derive(Clone, Debug)]
pub struct Trait {
    pub name: String,
    pub functions: Vec<Function>,
}

#[derive(Clone, Debug)]
pub struct Variable {
    pub name: String,
    pub typ: DeclarationContainer,
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
pub struct Function {
    pub name: String,
    pub arguments: Vec<(String, DeclarationContainer)>,
    pub return_type: DeclarationContainer,
    pub blocks: Vec<Arc<Mutex<Block>>>,
}

impl Function {
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
pub struct Behaviour {
    pub name: String,
    pub arguments: Vec<(String, DeclarationContainer)>,
    pub blocks: Vec<Arc<Mutex<Block>>>,
}

#[derive(Clone, Debug)]
pub struct Block {
    pub instructions: Vec<Arc<Mutex<Instruction>>>,
}

impl Block {
    pub fn new() -> Block {
        Block {
            instructions: Vec::with_capacity(5),
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
    GetParameter(Box<GetParameter>),
    FunctionCall(Box<FunctionCall>),
    Return(Box<Return>),
    Jump(Box<Jump>),
    Branch(Box<Branch>),
}

impl Instruction {
    pub fn get_type(&self) -> Box<Type> {
        return match self {
            Instruction::IntegerLiteral(_) => builtin::COMPTIME_INT.get_type(),
            Instruction::DeclarationReference(s) => s.declaration.get_type(),
            Instruction::GetParameter(_) => Box::new(types::Type::Unresolved(types::UnresolvedType { name: "UnimplementedParamGet".to_string() })),
            Instruction::FunctionCall(f) => {
                let ins = f.function.lock().unwrap();
                match *ins {
                    Instruction::DeclarationReference(ref f) => {
                        let guard = f.declaration.0.lock().unwrap();
                        if let Some(ref dec) = *guard {
                            match **dec {
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
    pub fn get_type_with_context(&self, context: Either<&Box<Function>, &Box<Behaviour>>) -> Box<Type> {
        match self {
            Instruction::GetParameter(g) => {
                let name = &g.name;
                match context {
                    Either::Left(f) => {
                        for (arg_name, declaration) in &f.arguments {
                            if arg_name == name {
                                return declaration.get_type();
                            }
                        }
                    }
                    Either::Right(b) => {
                        for (arg_name, declaration) in &b.arguments {
                            if arg_name == name {
                                return declaration.get_type();
                            }
                        }
                    }
                }
            }
            _ => {}
        }
        self.get_type()
    }
}

// -- INSTRUCTIONS -- \\

#[derive(Clone, Debug)]
pub struct IntegerLiteral(pub i64);

#[derive(Clone, Debug)]
pub struct DeclarationReference {
    pub name: (Option<ast::Path>, String),
    pub declaration: DeclarationContainer,
}

impl DeclarationReference {
    pub fn blank_with_path(path: ast::Path, name: String) -> Self {
        Self {
            name: (Some(path), name),
            declaration: DeclarationContainer::empty(),
        }
    }

    pub fn blank(name: String) -> Self {
        Self {
            name: (None, name),
            declaration: DeclarationContainer::empty(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct GetParameter {
    pub name: String,
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
pub struct Jump {
    pub block: Arc<Mutex<Block>>,
}

#[derive(Clone, Debug)]
pub struct Branch {
    pub condition: Arc<Mutex<Instruction>>,
    pub true_block: Arc<Mutex<Block>>,
    pub false_block: Arc<Mutex<Block>>,
}
