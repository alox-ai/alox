const std = @import("std");
const ast = @import("ast.zig");
const ir = @import("ir.zig");

pub const allocator = std.heap.page_allocator;

pub fn main() anyerror!void {
    std.debug.warn("Alox testing...\n", .{});
    var x = ast.VariableReference {
        .path = try ast.Path.of("test"),
        .name = "a",
    };
    var path = try ast.Path.of("aaa");
    path = try path.append("bbb");
    path = try path.append("ccc");
    std.debug.warn("{}\n", .{path.toString()});

    _ = ir.Module;
}
