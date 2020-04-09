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
pub struct DeclarationId {
    pub path: ast::Path,
    pub name: String,
    pub arguments: Vec<Box<DeclarationId>>,
}

impl DeclarationId {
    pub fn from(module: Option<&Module>, declaration: &Declaration) -> Self {
        let declaration_id = declaration.name();
        if let Some(module) = module {
            let module_id = module.full_path();
            (module_id, declaration_id).into()
        } else {
            declaration_id.into()
        }
    }

    pub fn from_type_name(type_name: &ast::TypeName) -> Self {
        let mut arguments = Vec::new();
        for arg in type_name.arguments.iter() {
            arguments.push(Box::new(DeclarationId::from_type_name(arg)));
        }
        DeclarationId {
            path: type_name.path.clone(),
            name: type_name.name.clone(),
            arguments,
        }
    }

    pub fn get_type(&self, compiler: &Compiler) -> Box<types::Type> {
        match compiler.resolve(self) {
            Some(dec) =>
                dec.get_type(compiler),
            None => Box::new(types::Type::Unresolved(types::UnresolvedType::of(&format!("u*{}", self.name()))))
        }
    }

    pub fn name(&self) -> String {
        if self.arguments.len() > 0 {
            let mut string_arguments = Vec::new();
            for arg in self.arguments.iter() {
                string_arguments.push(arg.name());
            }
            let mut name = self.name.clone();
            name.push_str("[");
            name.push_str(&string_arguments.join(", "));
            name.push_str("]");
            name
        } else {
            self.name.clone()
        }
    }
}

impl From<(ast::Path, String)> for DeclarationId {
    fn from(pair: (ast::Path, String)) -> Self {
        Self {
            path: pair.0,
            name: pair.1,
            arguments: vec![],
        }
    }
}

impl From<String> for DeclarationId {
    fn from(name: String) -> Self {
        Self {
            path: ast::Path::new(),
            name,
            arguments: vec![],
        }
    }
}

pub struct Compiler {
    pub modules: RwLock<Vec<Module>>,
    pub declaration_bank: RwLock<HashMap<DeclarationId, usize>>,
    pub generated_declarations: RwLock<HashMap<DeclarationId, Declaration>>,
}

impl Compiler {
    pub fn new() -> Compiler {
        Compiler {
            modules: RwLock::new(Vec::with_capacity(5)),
            declaration_bank: RwLock::new(HashMap::new()),
            generated_declarations: RwLock::new(HashMap::new()),
        }
    }

    pub fn add_module(&self, module: Module) {
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
    ) -> Option<&Declaration> {
        let declaration_id = (path, name).into();
        self.resolve(&declaration_id)
    }

    pub fn resolve(&self, declaration_id: &DeclarationId) -> Option<&Declaration> {
        if declaration_id.path.0.is_empty() {
            match builtin::find_builtin_declaration(declaration_id) {
                Some(dec) => return Some(&dec),
                _ => {
                    if let Some(d) = self.generated_declarations.read().unwrap().get(declaration_id) {
                        // copy the pointer because it points into the map and is valid as long as the compiler reference is valid
                        let dec_ptr = d as *const Declaration;
                        return Some(unsafe { &*dec_ptr });
                    }

                    // these are builtin types that need to be dynamically generated, such as builtin generic types
                    let name = declaration_id.name.clone();
                    match name.as_str() {
                        "Pointer" | "Array" => {
                            let mut arguments = Vec::new();

                            for arg in declaration_id.arguments.iter() {
                                let arg_type = match self.resolve(arg) {
                                    Some(dec) => dec.get_type(self),
                                    None => Box::new(types::Type::Unresolved(types::UnresolvedType { name: "NoArgDec".to_string() })),
                                };
                                arguments.push(arg_type);
                            }

                            let type_dec = Declaration::Type(Box::new(types::Type::GenericType(types::GenericType {
                                name,
                                arguments,
                            })));

                            let dec_ptr = &type_dec as *const Declaration;
                            self.generated_declarations.write().unwrap().insert(declaration_id.clone(), type_dec);
                            return Some(unsafe { &*dec_ptr });
                        }
                        _ => {}
                    }
                }
            }
        }
        if let Some(declaration) = self.declaration_bank.read().unwrap().get(declaration_id) {
            let dec_ptr = *declaration as *const Declaration;
            Some(unsafe { &*dec_ptr })
        } else { None }
    }
}

#[derive(Clone, Debug)]
pub struct Module {
    /// path doesn't contain the module's name
    pub path: ast::Path,
    pub name: String,
    pub declarations: Vec<Declaration>,
}

impl Module {
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

    pub fn get_type(&self, compiler: &Compiler) -> Box<Type> {
        match self {
            Declaration::Function(f) => f.get_type(compiler),
            Declaration::Struct(s) => s.get_type(compiler),
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
    pub fields: Vec<Declaration>,
    // Declaration::Function
    pub functions: Vec<Declaration>,
    // Declaration::Behaviour
    pub behaviours: Vec<Declaration>,
}

#[derive(Clone, Debug)]
pub struct Struct {
    pub name: String,
    // Declaration::Variable
    pub fields: Vec<Declaration>,
    // Declaration::Trait
    pub traits: Vec<Declaration>,
    // Declaration::Function
    pub functions: Vec<Declaration>,
}

impl Struct {
    pub fn get_type(&self, compiler: &Compiler) -> Box<types::Type> {
        let mut fields = Vec::with_capacity(self.fields.len());
        for field in self.fields.iter() {
            let name = field.name();
            let typ = field.get_type(compiler);
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
    pub mutable: bool,
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
pub struct Function {
    pub name: String,
    pub arguments: Vec<(String, DeclarationId)>,
    pub return_type: DeclarationId,
    pub blocks: Vec<Block>,
}

impl Function {
    pub fn get_type(&self, compiler: &Compiler) -> Box<types::Type> {
        let mut arguments = Vec::<Box<types::Type>>::with_capacity(self.arguments.len());

        for arg in &self.arguments {
            arguments.push(arg.1.get_type(compiler));
        }

        let result = self.return_type.get_type(compiler);
        Box::new(types::Type::Function(types::FunctionType { arguments, result }))
    }
}

#[derive(Clone, Debug)]
pub struct Behaviour {
    pub name: String,
    pub arguments: Vec<(String, DeclarationId)>,
    pub blocks: Vec<Block>,
}

#[derive(Clone, Debug)]
pub struct Block {
    pub id: BlockId,
    pub instructions: Vec<Instruction>,
}

impl Block {
    pub fn new(id: usize) -> Block {
        Block {
            id: BlockId(id),
            instructions: Vec::with_capacity(5),
        }
    }

    pub fn add_instruction(&mut self, instruction: Instruction, compiler: &Compiler) -> InstructionId {
        // don't add the instruction to this block if it already has an instruction
        // that doesn't return, like Return, Branch, Jump, etc
        {
            let mut found = false;
            let mut found_ins = 0;
            for (index, instruction) in self.instructions.iter().enumerate() {
                match *instruction.get_type(compiler, self) {
                    Type::Primitive(PrimitiveType::NoReturn) => {
                        found_ins = index;
                        found = true;
                        break;
                    }
                    _ => {}
                }
            }
            if found {
                return InstructionId(found_ins);
            }
        }
        self.instructions.push(instruction);
        InstructionId(self.instructions.len() - 1)
    }

    pub fn get_instruction(&self, id: InstructionId) -> &Instruction {
        self.instructions.get(id.0 as usize).expect("invalid instruction id")
    }
}

#[derive(Clone, Debug)]
pub enum Instruction {
    Unreachable(String),
    Load(Box<Load>),
    Store(Box<Store>),
    Alloca(Box<Alloca>),
    BooleanLiteral(Box<BooleanLiteral>),
    IntegerLiteral(Box<IntegerLiteral>),
    DeclarationReference(Box<DeclarationReference>),
    GetParameter(Box<GetParameter>),
    FunctionCall(Box<FunctionCall>),
    Return(Box<Return>),
    Jump(Box<Jump>),
    Branch(Box<Branch>),
}

impl Instruction {
    pub fn get_type(&self, compiler: &Compiler, block: &Block) -> Box<Type> {
        return match self {
            Instruction::BooleanLiteral(_) => builtin::BOOL.get_type(compiler),
            Instruction::IntegerLiteral(_) => builtin::COMPTIME_INT.get_type(compiler),
            Instruction::DeclarationReference(s) => s.declaration.get_type(compiler),
            Instruction::GetParameter(_) => Box::new(types::Type::Unresolved(types::UnresolvedType { name: "UnimplementedParamGet".to_string() })),
            Instruction::FunctionCall(f) => {
                let func_ins_id = f.function;
                let ins = block.get_instruction(func_ins_id);
                match *ins {
                    Instruction::DeclarationReference(ref f) => {
                        if let Some(ref dec) = compiler.resolve(&f.declaration) {
                            match *dec {
                                Declaration::Function(ref h) => h.return_type.get_type(compiler),
                                _ => Box::new(types::Type::Unresolved(types::UnresolvedType { name: "UnPointerToFuncBody".to_string() }))
                            }
                        } else {
                            Box::new(types::Type::Unresolved(types::UnresolvedType { name: "UnNoFunctionDec".to_string() }))
                        }
                    }
                    _ => Box::new(types::Type::Unresolved(types::UnresolvedType { name: "UnDecNotFunc".to_string() }))
                }
            }
            Instruction::Alloca(alloca) => {
                Box::new(types::Type::GenericType(types::GenericType { name: "Pointer".to_string(), arguments: vec![alloca.typ.clone()] }))
            }
            Instruction::Load(load) => {
                load.typ.clone()
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
    pub fn get_type_with_context(&self, compiler: &Compiler, block: &Block, context: Either<&Box<Function>, &Box<Behaviour>>) -> Box<Type> {
        match self {
            Instruction::GetParameter(g) => {
                let name = &g.name;
                match context {
                    Either::Left(f) => {
                        for (arg_name, declaration) in &f.arguments {
                            if arg_name == name {
                                return if let Some(declaration) = compiler.resolve(declaration) {
                                    declaration.get_type(compiler)
                                } else {
                                    declaration.get_type(compiler)
                                };
                            }
                        }
                    }
                    Either::Right(b) => {
                        for (arg_name, declaration) in &b.arguments {
                            if arg_name == name {
                                return if let Some(declaration) = compiler.resolve(declaration) {
                                    declaration.get_type(compiler)
                                } else {
                                    declaration.get_type(compiler)
                                };
                            }
                        }
                    }
                }
            }
            _ => {}
        }
        self.get_type(compiler, block)
    }
}

// -- INSTRUCTIONS -- \\

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct BlockId(pub usize);

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct InstructionId(pub usize);

#[derive(Clone, Debug)]
pub struct Load {
    pub name: String,
    pub typ: Box<Type>,
}

#[derive(Clone, Debug)]
pub struct Store {
    pub name: String,
    pub value: InstructionId,
}

#[derive(Clone, Debug)]
pub struct Alloca {
    pub name: String,
    pub typ: Box<Type>,
}

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
pub struct FunctionCall {
    pub function: InstructionId,
    pub arguments: Vec<InstructionId>,
}

#[derive(Clone, Debug)]
pub struct Return {
    pub instruction: InstructionId,
}

#[derive(Clone, Debug)]
pub struct Jump {
    pub block: BlockId,
}

#[derive(Clone, Debug)]
pub struct Branch {
    pub condition: InstructionId,
    pub true_block: BlockId,
    pub false_block: BlockId,
}
