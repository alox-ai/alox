#[derive(Clone, Debug)]
pub enum Type {
    Unresolved(String),
    Primitive(PrimitiveType),
}

impl Type {
    pub fn name(&self) -> String {
        match self {
            Type::Unresolved(s) => s.clone(),
            Type::Primitive(p) => p.name(),
        }
    }
}

#[derive(Clone, Debug)]
pub enum PrimitiveType {
    Int(u8),
    Float(u8),
    Bool,
}

impl PrimitiveType {
    pub fn name(&self) -> String {
        match self {
            PrimitiveType::Int(size) => format!("Int{}", *size),
            PrimitiveType::Float(size) => format!("Float{}", *size),
            PrimitiveType::Bool => String::from("Bool")
        }
    }
}
