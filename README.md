# alox

> Systems Programming meets Verified Programming

This is very much a _Work In Progress_, nothing works yet.

Here's a roadmap:

* Frontend
    * [x] Lexer
    * [ ] Parser
    * [ ] Start parsing imported modules immediately
* Middle
    * [x] AST Structure
    * [x] Thread-safe IR Structure
    * [x] Concurrent IR Symbol Resolution
    * [x] AST Expression -> IR Instruction conversion
    * [ ] Passes to validate things
* Error messages
    * [x] Parser error messages
    * [ ] Missing declaration messages
    * [ ] Validation messages
* Backend
    * [ ] Look at [CraneLift](https://github.com/CraneStation/CraneLift)
    * [ ] LLVM ([inkwell](https://github.com/TheDan64/inkwell) or [llvm-sys](https://crates.io/crates/llvm-sys)
* Really dig into semantics
---

Language Ideas

* Pure code by default
    * Allows for compile time code execution
    * Permission based system - `+IO, +Syscall, +MutateArgs`
* Strong type system
    * Algebraic Data Types
    * Refinements on function arguments
    * Unique & Borrowed Types
* Automatic Versioning
    * Enforce public APIs
* Clean syntax
* **Concurrent compiler pipeline**

```rust
import std::io

let INT32_MAX: Int32 = 2_147_483_647

fun bounded(n: Int32): Bool {
    return (addWithOverflow(n, INT32_MAX) > 0) && (n < INT32_MAX)
}

// add function that can't overflow at runtime
// 'bounded' can be used because it is a pure function
fun add(x: Int32, y: Int32): Int32
  where (y: bounded(x + y), return: x + y)  {
    return a + b
}

import std.io

// println requires the caller to be annotated with +IO
fun main() +IO {
    let a = INT32_MAX - 2
    let b = 3
    // compile time error!
    let c = add(a, b)
    std::io::println(c)
}
```

```rust
import std::io

trait Action {
    fun action(): Int32;
    fun otherAction(): Int32 +MutateSelf;
}

struct Container : Action {
    let x: Int32
    let y: Int32

    fun action(): Int32 {
        return self.x + self.y
    }
    
    // this function is allowed to mutate itself
    fun otherAction(): Int32 +MutateSelf -> {
        self.x = self.x + 1
        return self.x
    }
}

fun main() +IO {
    let container: Action = Container { x: 1, y: 2 }
    container.action()
    container.otherAction()
    std::io::println()
}
```

Inspired by _Rust, Liquid Haskell, & many more_.
