# alox prototype

Ideas

* Pure code by default
    * Allows for compile time code execution
    * Permission based system - `+IO, +Syscall, +MutateArgs`
* Strong type system
    * Algebraic Data Types
    * Refinement Types
* Automatic Versioning
    * Enforce public APIs
* Clean syntax
* Concurrent compiler pipeline

```rust
let INT32_MAX: Int32 = 2_147_483_647

fun bounded(n: Int32): Bool
let bounded = (n) -> { 
    return (addWithOverflow(n, INT32_MAX) > 0) && (n < INT32_MAX)
}

// add function that can't overflow at runtime
// 'bounded' can be used because it is a pure function
fun add(x: Int32, y: Int32): Int32
    where (y: bounded(x + y), return: x + y)
let add = (a, b) -> {
    return a + b
}

import std.io

// println requires the caller to be annotated with +IO
fun main() +IO
let main = () -> {
    let a = INT32_MAX - 2
    let b = 3
    // compile time error!
    let c = add(a, b)
    io.println(c)
}
```

```rust
trait Action {
    fun action(): Int32
    fun otherAction(): Int32 +MutateSelf
}

struct Container : Action {
    let x: Int32
    let y: Int32
    
    // function declaration is taken from the trait
    let action = () -> {
        return self.x + self.y
    }
    
    // this function is allowed to mutate itself
    let otherAction = () -> {
        self.x = self.x + 1
        return self.x
    }
}

fun main() +IO
let main = () -> {
    let container: Action = Container { x: 1, y: 2 }
    container.action()
    container.otherAction()
    io.println()
}
```

Inspired by _Rust, Liquid Haskell, Kitten, & many more_.
