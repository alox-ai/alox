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
    Int32,
    Bool,
}

impl PrimitiveType {
    pub fn name(&self) -> String {
        match self {
            PrimitiveType::Int32 => String::from("Int32"),
            PrimitiveType::Bool => String::from("Bool")
        }
    }
}
