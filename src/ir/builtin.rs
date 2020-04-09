use crate::ir::DeclarationId;
use crate::ir::Declaration;
use crate::ir::types::PrimitiveType;
use crate::ir::types::Type;

pub fn find_builtin_declaration(declaration_id: &DeclarationId) -> Option<&'static Declaration> {
    match declaration_id.name.as_str() {
        "Int8" => Some(&*INT8),
        "Int16" => Some(&*INT16),
        "Int32" => Some(&*INT32),
        "Int64" => Some(&*INT64),
        "ComptimeInt" => Some(&*COMPTIME_INT),
        "Bool" => Some(&*BOOL),
        "Float32" => Some(&*FLOAT32),
        "Float64" => Some(&*FLOAT64),
        "ComptimeFloat" => Some(&*COMPTIME_FLOAT),
        "NoReturn" => Some(&*NORETURN),
        "Void" => Some(&*VOID),
        _ => None
    }
}

fn wrap_primitive(typ: PrimitiveType) -> Declaration {
    Declaration::Type(Box::new(Type::Primitive(typ)))
}

// constant declarations for builtin types
lazy_static! {
    pub static ref INT8: Declaration = wrap_primitive(PrimitiveType::Int(8));
    pub static ref INT16: Declaration = wrap_primitive(PrimitiveType::Int(16));
    pub static ref INT32: Declaration = wrap_primitive(PrimitiveType::Int(32));
    pub static ref INT64: Declaration = wrap_primitive(PrimitiveType::Int(64));
    pub static ref COMPTIME_INT: Declaration = wrap_primitive(PrimitiveType::Int(255));

    pub static ref BOOL: Declaration = wrap_primitive(PrimitiveType::Bool);

    pub static ref FLOAT32: Declaration = wrap_primitive(PrimitiveType::Float(32));
    pub static ref FLOAT64: Declaration = wrap_primitive(PrimitiveType::Float(64));
    pub static ref COMPTIME_FLOAT: Declaration = wrap_primitive(PrimitiveType::Float(255));

    pub static ref NORETURN: Declaration = wrap_primitive(PrimitiveType::NoReturn);
    pub static ref VOID: Declaration = wrap_primitive(PrimitiveType::Void);
}
