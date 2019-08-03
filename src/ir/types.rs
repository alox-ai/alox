use core::fmt::{Formatter, Result};
use std::fmt::Debug;

pub trait Type: Send + Sync {
    fn name(&self) -> String;
}

impl Debug for Type {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "{}", self.name())?;
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

impl Type for UnresolvedType {
    fn name(&self) -> String {
        self.name.clone()
    }
}

pub struct StructType {
    pub name: String,
}

impl Type for StructType {
    fn name(&self) -> String {
        self.name.clone()
    }
}

pub struct FunctionType {
    pub arguments: Vec<Box<Type>>,
    pub result: Box<Type>,
}

impl Type for FunctionType {
    fn name(&self) -> String {
        let mut s = "".to_string();
        for x in &self.arguments {
            s.push_str(&x.name());
            s.push_str(" -> ")
        }
        s.push_str(&self.result.name());
        s
    }
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

impl Type for PrimitiveType {
    fn name(&self) -> String {
        match self {
            PrimitiveType::Int(size) => format!("Int{}", *size),
            PrimitiveType::Float(size) => format!("Float{}", *size),
            PrimitiveType::Bool => String::from("Bool"),
            PrimitiveType::Void => String::from("Void"),
            PrimitiveType::NoReturn => String::from("NoReturn")
        }
    }
}
