package dev.alox.compiler

import dev.alox.compiler.ast.Path
import dev.alox.compiler.backend.LLVMBackend
import dev.alox.compiler.ir.IrCompiler
import dev.alox.compiler.ir.PrettyPrinter
import dev.alox.compiler.ir.Translator
import dev.alox.compiler.ir.pass.SemanticAnalysis
import dev.alox.compiler.ir.pass.applyPasses
import dev.alox.compiler.parser.AstParser

fun main(args: Array<String>) {
    val parseResult = AstParser.parseModule(
        Path(listOf("alox")), "parsed", """
struct Box {
    let x : Int32
    let y : Int32
}

fun bar(): Int32 {
    let x: Int32 = 1
    return x
}

fun foo(box: Ref[Box]): Int32 {
    return box.x
}

fun baz(box: Box): Int32 {
    return box.x
}

fun foo2(box: Ref[Ref[Box]]): Int32 {
    return box.y
}

fun qux(box: Ref[Box]): Int32 {
    return foo(box)
}

fun quack(box: Ref[Box]): Int32 {
    if (true) {
        return box.x
    }
    return box.y
}

fun quack2(box: Ref[Box]): Int32 {
    if (true) {
        return box.x
    } else {
        return box.y
    }
}

actor A {
    let state: Int32

    behave ping(n: Int32, b: Ref[B]) {
        b.pong(n, this)
    }
}


actor B {
    let state: Int32
    
    behave pong(n: Int32, a: Ref[A]) {
        a.ping(n, this)
    }
}
    """.trimIndent()
    )

    if (parseResult is Either.Error) {
        println(parseResult.error.toString())
        return
    }
    val parsedModule = (parseResult as Either.Value).value

    val parsedIrModule = Translator(parsedModule).generateModule()

    val diagnostics = applyPasses(parsedIrModule, listOf(SemanticAnalysis))
    if (diagnostics.isNotEmpty()) {
        diagnostics.forEach { println(it) }
        return
    }

    PrettyPrinter(parsedIrModule)

    val compiler = IrCompiler()
    compiler.addModule(parsedIrModule)
    val llvm = LLVMBackend(compiler, parsedIrModule)
    llvm.process()
    llvm.dump()
}
