use core::fmt::{Formatter, Result};
use std::fmt::Debug;

#[derive(Clone)]
pub enum Type {
    Unresolved(UnresolvedType),
    Struct(StructType),
    Function(FunctionType),
    Primitive(PrimitiveType),
    GenericType(GenericType),
}

impl Type {
    pub fn name(&self) -> String {
        match self {
            Type::Unresolved(u) => u.name.clone(),
            Type::Struct(s) => s.name.clone(),
            Type::Function(f) => {
                let mut s = "".to_string();
                for x in &f.arguments {
                    s.push_str(&x.name());
                    s.push_str(" -> ")
                }
                s.push_str(&f.result.name());
                s
            }
            Type::Primitive(p) => {
                match p {
                    PrimitiveType::Int(size) =>
                        if *size < 255u8 { format!("Int{}", *size) } else { "ComptimeInt".to_string() },
                    PrimitiveType::Float(size) =>
                        if *size < 255u8 { format!("Float{}", *size) } else { "ComptimeFloat".to_string() },
                    PrimitiveType::Bool => String::from("Bool"),
                    PrimitiveType::Void => String::from("Void"),
                    PrimitiveType::NoReturn => String::from("NoReturn")
                }
            }
            Type::GenericType(g) => {
                let mut name = g.name.clone();
                name.push_str("[");
                for arg in g.arguments.iter() {
                    name.push_str(&arg.name());
                }
                name.push_str("]");
                name
            }
        }
    }
}

impl Debug for Type {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "{}", self.name())?;
        match self {
            Type::Unresolved(_) => { let _ = write!(f, "*"); }
            _ => {}
        }
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct UnresolvedType {
    pub name: String,
}

impl UnresolvedType {
    pub fn of(name: &str) -> Self {
        UnresolvedType { name: name.to_string() }
    }
}

#[derive(Clone, Debug)]
pub struct StructType {
    pub name: String,
    pub fields: Vec<(String, Box<Type>)>,
}

#[derive(Clone, Debug)]
pub struct FunctionType {
    pub arguments: Vec<Box<Type>>,
    pub result: Box<Type>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PrimitiveType {
    Int(u8),
    Float(u8),
    Bool,
    Void,
    NoReturn,
}

impl PrimitiveType {
    pub fn from_name(name: String) -> Option<PrimitiveType> {
        if name.starts_with("Int") {
            return Some(PrimitiveType::Int(name[3..].parse::<u8>().unwrap()));
        }
        if name.starts_with("Float") {
            return Some(PrimitiveType::Float(name[5..].parse::<u8>().unwrap()));
        }

        if name == "ComptimeInt".to_string() {
            return Some(PrimitiveType::Int(255));
        }
        if name == "ComptimeFloat".to_string() {
            return Some(PrimitiveType::Float(255));
        }
        if name == "Bool".to_string() {
            return Some(PrimitiveType::Bool);
        }
        if name == "Void".to_string() {
            return Some(PrimitiveType::Void);
        }
        if name == "NoReturn".to_string() {
            return Some(PrimitiveType::NoReturn);
        }
        None
    }
}

#[derive(Clone, Debug)]
pub struct GenericType {
    pub name: String,
    pub arguments: Vec<Box<Type>>,
}
