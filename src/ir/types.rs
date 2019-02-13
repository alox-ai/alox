#[derive(Clone, Debug)]
pub enum Type {
    Unresolved(String),
    Primitive(PrimitiveType),
}

#[derive(Clone, Debug)]
pub enum PrimitiveType {
    Int32,
    Bool,
}
