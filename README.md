# alox

> GPU Accelerated, Distributed, Actor Model Language

Goals:

* Have code running on the GPU and CPU
* Have code running across many machines
* Use the actor model for concurrency

This is very much a _Work In Progress_, nothing works yet.

Roadmap:

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
    * [ ] Validation messages
* Backend
    * Normal Backends
        * [ ] Look at [CraneLift](https://github.com/CraneStation/CraneLift)
        * [ ] LLVM ([inkwell](https://github.com/TheDan64/inkwell) or [llvm-sys](https://crates.io/crates/llvm-sys))
    * GPU Backends
        * [ ] Dynamically figure out the best backend to use
        * [ ] SPIR-V ([rspirv](https://github.com/gfx-rs/rspirv))
        * [ ] CUDA ([libcuda](https://github.com/peterhj/libcuda))
        * [ ] OpenCL (for older platforms?) ([ocl](https://github.com/cogciprocate/ocl))
* Runtime
    * [ ] Schedulers
    * [ ] Cross-node communication
    * [ ] GC for actors?
* Really dig into semantics
---

Language Ideas

* Compile time code execution
* Strong type system
    * Algebraic Data Types
    * Unique & Borrowed Types
* Automatic Versioning
    * Enforce public APIs
* Clean syntax
* Concurrent compiler pipeline

```pony
actor A {
    behave ping(n: Int32, b: &B) {
        b.pong(n, &this)
    }
}

actor B {
    behave pong(n: Int32, a: &A) {
        let x = n & 0xF0 >> 4
        let y = n & 0x0F
        let arr = [x, y]
        let newArr = process(arr)
        let z = (newArr[0] << 4) | newArr[1]
        a.ping(z, &this)
    }

    fun process(arr: &mut [Int32]) {
        for (x in arr) {
            x *= 17 + (2 * x)
        }
    }
}

actor Main {
    behave main() {
        let n = 2
        let a = new A()
        let b = new B()
        a.ping(n, &b)
    }
}
```
