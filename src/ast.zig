const std = @import("std");
const main = @import("main.zig");

pub const List = std.ArrayList;

pub const Path = struct {
    const Self = @This();

    parts: List([]const u8),

    pub fn new() !Self {
        return Self {
            .parts = try List([]const u8).initCapacity(main.allocator, 3),
        };
    }

    pub fn of(part: []const u8) !Self {
        var path = try new();
        try path.parts.append(part);
        return path;
    }

    // clones the current path and appends `part` to it
    pub fn append(self: *Self, part: []const u8) anyerror!Path {
        var new_path = try self.clone();
        try new_path.parts.append(part);
        return new_path;
    }

    pub fn clone(self: *Self) !Self {
        var other = try new();
        for (self.parts.toSlice()) |part| {
            try other.parts.append(part);
        }
        return other;
    }

    /// caller is responsible for freeing the result
    pub fn toString(self: *Self) ![]const u8 {
        // count the bytes needed including "::"
        var size: usize = 0;
        for (self.parts.toSlice()) |part| {
            size += part.len + 2;
        }
        size -= 2;
        // allocate the space needed for the buffer
        var buffer = try main.allocator.alloc(u8, size);

        // go through each part and append it to the buffer
        var inserted_bytes: usize = 0;
        for (self.parts.toSlice()) |part| {
            for (part) |byte| {
                buffer[inserted_bytes] = byte;
                inserted_bytes += 1;
            }
            if (inserted_bytes < size) {
                buffer[inserted_bytes] = ':';
                buffer[inserted_bytes + 1] = ':';
                inserted_bytes += 2;
            }
        }
        return buffer;
    }
};

pub const NamespacedIdentifier = struct {
    path: Path,
    name: []const u8,
};

pub const Program = struct {
    path: Path,
    file_name: String,
    imports: List(Path),
    nodes: List(Node),
};

// nodes

pub const Node = union(enum) {
    ActorType: Actor,
    StructType: Struct,
    FunctionType: Function,
    VariableDeclarationType: VariableDeclaration,
};

pub const Struct = struct {
    nae: []const u8,
    fields: List(VariableDeclaration),
    functions: List(Function),
};

pub const Actor = struct {
    name: []const u8,
    fields: List(VariableDeclaration),
    functions: List(Function),
    behaviours: List(Behaviour),
};

pub const Argument = struct {
    name: []const u8,
    type_name: NamespacedIdentifier,
};

pub const Function = struct {
    name: []const u8,
    arguments: List(Argument),
    return_type: NamespacedIdentifier,
    statements: List(Statement),
};

pub const Behaviour = struct {
    name: []const u8,
    arguments: List(Argument),
    statements: List(Statement),
};

// statements

pub const Statement = union(enum) {
    VariableDeclarationType: VariableDeclaration,
    IfType: If,
    ReturnType: Return,
    FunctionCallType: FunctionCall,
};

pub const VariableDeclaration = struct {
    name: []const u8,
    type_name: ?NamespacedIdentifier,
    initial_expression: ?Expression,
};

pub const Return = struct {
    expression: Expression,
};

pub const If = struct {
    condition: Expression,
    block: List(Statement),
    elseif: ?If,
};

// expressions

pub const Expression = union(enum) {
    BooleanLiteralType: BooleanLiteral,
    IntegerLiteralType: IntegerLiteralType,
    VariableReferenceType: VariableReference,
    FunctionCallType: FunctionCall,
};

pub const VariableReference = struct {
    path: ?Path,
    name: []const u8,
};

pub const BooleanLiteral = struct {
    value: bool,
};

pub const IntegerLiteral = struct {
    value: i64,
};

pub const FunctionCall = struct {
    function: Expression,
    arguments: List(Expression),
};
