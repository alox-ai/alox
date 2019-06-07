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
    name: String,
}

impl Type for UnresolvedType {
    fn name(&self) -> String {
        self.name.clone()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PrimitiveType {
    Int(u8),
    Float(u8),
    Bool,
    Void,
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
        }
    }
}
