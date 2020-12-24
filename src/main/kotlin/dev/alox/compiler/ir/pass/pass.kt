package dev.alox.compiler.ir.pass

import dev.alox.compiler.Either
import dev.alox.compiler.ir.IrModule
import dev.alox.compiler.report.Diagnostic

abstract class Pass {

    val diagnostics: MutableList<Diagnostic> = mutableListOf()

    fun accept(module: IrModule) {
        module.declarations.forEach { acceptDeclaration(module, it) }
    }

    open fun acceptDeclaration(
        module: IrModule,
        declaration: IrModule.Declaration,
        parentStruct: IrModule.Declaration.Struct? = null
    ) {
        when (declaration) {
            is IrModule.Declaration.Struct -> {
                declaration.declarations.forEach { acceptDeclaration(module, it, declaration) }
            }
            is IrModule.Declaration.Function -> {
                acceptFunction(module, declaration, parentStruct)
            }
        }
    }

    open fun acceptFunction(
        module: IrModule,
        function: IrModule.Declaration.Function,
        parentStruct: IrModule.Declaration.Struct? = null
    ) {

    }

}

fun applyPasses(module: IrModule, passes: List<Pass>): List<Diagnostic> {
    return passes.flatMap {
        it.accept(module)
        it.diagnostics
    }
}
