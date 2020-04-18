use std::collections::HashMap;
use std::fmt::{Display, Error, Formatter};
use std::sync::RwLock;

use codespan_reporting::diagnostic::Diagnostic;

use crate::ast;
use crate::diagnostic::{DiagnosticManager, FileId};
use crate::ir::pass::PassManager;
use crate::ir::types::{PrimitiveType, Type};
use crate::parser::Parser;

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
    pub diagnostics: RwLock<DiagnosticManager>,
}

impl Compiler {
    pub fn new() -> Compiler {
        Compiler {
            modules: RwLock::new(Vec::with_capacity(5)),
            declaration_bank: RwLock::new(HashMap::new()),
            generated_declarations: RwLock::new(HashMap::new()),
            diagnostics: RwLock::new(DiagnosticManager::new()),
        }
    }

    pub fn compile(&self, path: ast::Path, file_name: String, source: String) -> Result<(), String> {
        let mut parser = Parser::new();
        if let Some(program) = parser.parse(path, file_name, source) {
            // import the diagnostics from the parser
            self.copy_diagnostics(parser.diagnostics);
            if self.diagnostics.read().unwrap().has_errors() {
                Err(String::from("Failed to compile module"))
            } else {
                // generate the module and add it
                let module = self.generate_ir(program);
                self.add_module(module);
                Ok(())
            }
        } else {
            Err(String::from("Failed to compile module"))
        }
    }

    pub fn copy_diagnostics(&self, mut other: DiagnosticManager) {
        let mut diagnostics = self.diagnostics.write().unwrap();
        for (file_name, id) in other.file_ids.iter() {
            let source = other.files.get(*id).unwrap().source();
            diagnostics.add_file(file_name.to_string(), source.to_string());
        }
        diagnostics.messages.append(&mut other.messages);
    }

    pub fn add_diagnostic(&self, diagnostic: Diagnostic<FileId>) {
        self.diagnostics.write().unwrap().add_diagnostic(diagnostic);
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

#[derive(Clone, Debug)]
pub enum Declaration {
    Function(Box<Function>),
    Struct(Box<Struct>),
    Trait(Box<Trait>),
    Variable(Box<Variable>),
    Type(Box<Type>),
}

impl Declaration {
    pub fn name(&self) -> String {
        match self {
            Declaration::Function(f) => f.name.clone(),
            Declaration::Struct(s) => s.name.clone(),
            Declaration::Trait(t) => t.name.clone(),
            Declaration::Variable(v) => v.name.clone(),
            Declaration::Type(t) => t.name(),
        }
    }

    pub fn get_type(&self, compiler: &Compiler) -> Box<Type> {
        match self {
            Declaration::Function(f) => f.get_type(compiler),
            Declaration::Struct(s) => s.get_type(compiler),
            Declaration::Type(t) => t.clone(),
            _ => Box::new(types::Type::Unresolved(types::UnresolvedType::of("UnresolvedDeclaration"))),
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum StructKind {
    Struct,
    Actor,
}

impl StructKind {
    pub fn name(&self) -> &str {
        match self {
            StructKind::Struct => "struct",
            StructKind::Actor => "actor",
        }
    }
}

impl From<ast::StructKind> for StructKind {
    fn from(other: ast::StructKind) -> Self {
        match other {
            ast::StructKind::Struct => StructKind::Struct,
            ast::StructKind::Actor => StructKind::Actor,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Struct {
    pub kind: StructKind,
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

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum FunctionKind {
    Function,
    Behaviour,
    Kernel,
}

impl FunctionKind {
    pub fn name(&self) -> &str {
        match self {
            FunctionKind::Function => "fun",
            FunctionKind::Behaviour => "behave",
            FunctionKind::Kernel => "kernel",
        }
    }
}

impl From<ast::FunctionKind> for FunctionKind {
    fn from(other: ast::FunctionKind) -> Self {
        match other {
            ast::FunctionKind::Function => FunctionKind::Function,
            ast::FunctionKind::Behaviour => FunctionKind::Behaviour,
            ast::FunctionKind::Kernel => FunctionKind::Kernel,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Function {
    pub kind: FunctionKind,
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

    pub fn is_function(&self) -> bool {
        self.kind == FunctionKind::Function
    }

    pub fn is_behaviour(&self) -> bool {
        self.kind == FunctionKind::Behaviour
    }

    pub fn is_kernel(&self) -> bool {
        self.kind == FunctionKind::Kernel
    }
}

#[derive(Clone, Debug)]
pub struct Block {
    pub id: BlockId,
    pub ins_start_offset: usize,
    pub instructions: Vec<Instruction>,
}

impl Block {
    pub fn new(id: usize, ins_start: usize) -> Block {
        Block {
            id: BlockId(id),
            ins_start_offset: ins_start,
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
                return InstructionId(found_ins + self.ins_start_offset);
            }
        }
        self.instructions.push(instruction);
        InstructionId(self.instructions.len() - 1 + self.ins_start_offset)
    }

    pub fn get_instruction(&self, id: InstructionId) -> &Instruction {
        self.instructions.get(id.0 as usize - self.ins_start_offset).expect("invalid instruction id")
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
            Instruction::IntegerLiteral(_) => builtin::INT32.get_type(compiler),
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
                let inner_type = block.get_instruction(alloca.reference_ins).get_type(compiler, block);
                types::GenericType::wrap("Pointer".into(), inner_type)
            }
            Instruction::Load(load) => {
                block.get_instruction(load.reference_ins).get_type(compiler, block)
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
    pub fn get_type_with_context(&self, compiler: &Compiler, block: &Block, context: &Function) -> Box<Type> {
        if let Instruction::GetParameter(g) = self {
            let name = &g.name;
            for (arg_name, declaration) in &context.arguments {
                if arg_name == name {
                    return if let Some(declaration) = compiler.resolve(declaration) {
                        declaration.get_type(compiler)
                    } else {
                        declaration.get_type(compiler)
                    };
                }
            }
        }
        if let Instruction::Load(l) = self {
            for block in context.blocks.iter() {
                for (ins_id, ins) in block.instructions.iter().enumerate() {
                    let ins_id = ins_id + block.ins_start_offset;
                    if ins_id == l.reference_ins.0 {
                        return ins.get_type_with_context(compiler, block, context);
                    }
                }
            }
        }
        self.get_type(compiler, block)
    }
}

// -- INSTRUCTIONS -- \\

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct BlockId(pub usize);

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct InstructionId(pub usize);

#[derive(Clone, Debug)]
pub struct Load {
    pub name: String,
    pub reference_ins: InstructionId,
}

#[derive(Clone, Debug)]
pub struct Store {
    pub name: String,
    pub value: InstructionId,
}

#[derive(Clone, Debug)]
pub struct Alloca {
    pub name: String,
    pub reference_ins: InstructionId,
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
