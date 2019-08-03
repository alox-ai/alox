use crate::ast;
use crate::ir;
use crate::ir::{Declaration, DeclarationContainer};
use crate::ir::types::PrimitiveType;

pub fn find_builtin_declaration(name: String, kind: Option<ir::DeclarationKind>) -> Option<ir::DeclarationContainer> {
    if let Some(ir::DeclarationKind::Type) = kind {
        match name.as_str() {
            "Int8" => Some(INT8.clone()),
            "Int16" => Some(INT16.clone()),
            "Int32" => Some(INT32.clone()),
            "Int64" => Some(INT64.clone()),
            "ComptimeInt" => Some(COMPTIME_INT.clone()),
            "Bool" => Some(BOOL.clone()),
            "Float32" => Some(FLOAT32.clone()),
            "Float64" => Some(FLOAT64.clone()),
            "ComptimeFloat" => Some(COMPTIME_FLOAT.clone()),
            "NoReturn" => Some(NORETURN.clone()),
            "Void" => Some(VOID.clone()),
            _ => None
        }
    } else {
        None
    }
}

fn wrap_primitive(typ: PrimitiveType) -> DeclarationContainer {
    DeclarationContainer::from(Declaration::PrimitiveType(Box::new(typ)))
}

// constant declarations for builtin types
lazy_static! {
    pub static ref INT8: DeclarationContainer = wrap_primitive(PrimitiveType::Int(8));
    pub static ref INT16: DeclarationContainer = wrap_primitive(PrimitiveType::Int(16));
    pub static ref INT32: DeclarationContainer = wrap_primitive(PrimitiveType::Int(32));
    pub static ref INT64: DeclarationContainer = wrap_primitive(PrimitiveType::Int(64));
    pub static ref COMPTIME_INT: DeclarationContainer = wrap_primitive(PrimitiveType::Int(255));

    pub static ref BOOL: DeclarationContainer = wrap_primitive(PrimitiveType::Bool);

    pub static ref FLOAT32: DeclarationContainer = wrap_primitive(PrimitiveType::Float(32));
    pub static ref FLOAT64: DeclarationContainer = wrap_primitive(PrimitiveType::Float(64));
    pub static ref COMPTIME_FLOAT: DeclarationContainer = wrap_primitive(PrimitiveType::Float(255));

    pub static ref NORETURN: DeclarationContainer = wrap_primitive(PrimitiveType::NoReturn);
    pub static ref VOID: DeclarationContainer = wrap_primitive(PrimitiveType::Void);
}