const std = @import("std");
const main = @import("main.zig");
const ast = @import("ast.zig");

const List = std.ArrayList;

pub const DeclarationHole = struct {
    const Self = @This();

    dec: *?*Declaration,

    pub fn from(declaration: *Declaration) Self {
        return Self {.dec = &declaration};
    }

    pub fn empty() Self {
        return Self {.dec = &null};
    }
};

pub const Module = struct {
    const Self = @This();

    path: ast.Path,
    name: []const u8,
    declarations: List(Declaration),

    fn fullPath(self: *Self) ast.Path {
        return self.path.append(name);
    }
};

pub const Declaration = union(enum) {
    BehaviourType: Behaviour,
    FunctionType: Function,
    ActorType: Actor,
    StructType: Struct,
    VariableType: Variable,
    TypeType,//: Type,
};

pub const Actor = struct {
    name: []const u8,
    fields: List(DeclarationHole),
    functions: List(DeclarationHole),
    behaviours: List(DeclarationHole),
};

pub const Struct = struct {
    name: []const u8,
    fields: List(DeclarationHole),
    functions: List(DeclarationHole),
};

pub const Variable = struct {
    name: []const u8,
    typ: DeclarationHole,
};

pub const Argument = struct {
    name: []const u8,
    typ: DeclarationHole,
};

pub const Function = struct {
    name: []const u8,
    arguments: List(Argument),
    return_type: DeclarationHole,
    blocks: List(Block),
};

pub const Behaviour = struct {
    name: []const u8,
    arguments: List(Argument),
    blocks: List(Block),
};

pub const Block = struct {
    instructions: List(Instruction),
};

pub const Instruction = union(enum) {
    Unreachable,
    BooleanLiteralType: BooleanLiteral,
    IntegerLiteralType: IntegerLiteral,
    DeclarationReferenceType: DeclarationReference,
    GetParameterType: GetParameter,
    FunctionCallType: FunctionCall,
    ReturnType: Return,
    JumpType: Jump,
    BranchType: Branch,
};

pub const BooleanLiteral = struct {
    value: bool,
};

pub const IntegerLiteral = struct {
    value: i64,
};

pub const DeclarationReference = struct {
    path: ?ast.Path,
    name: []const u8,
    declaration: DeclarationHole,
};

pub const GetParameter = struct {
    name: []const u8,
};

pub const FunctionCall = struct {
    function: *Instruction,
    arguments: List(*Instruction),
};

pub const Return = struct {
    instruction: *Instruction,
};

pub const Jump = struct {
    block: *Block,
};

pub const Branch = struct {
    condition: *Instruction,
    true_block: *Block,
    false_block: *Block,
};
