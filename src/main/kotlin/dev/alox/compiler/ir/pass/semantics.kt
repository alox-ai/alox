package dev.alox.compiler.ir.pass

import dev.alox.compiler.ir.IrModule
import dev.alox.compiler.report.Diagnostic
import dev.alox.compiler.report.Label

object SemanticAnalysis : Pass() {

    override fun acceptFunction(
        module: IrModule,
        function: IrModule.Declaration.Function,
        parentStruct: IrModule.Declaration.Struct?
    ) {
        // this function is a behavior outside of an actor
        if (function.kind == IrModule.Declaration.Function.Kind.BEHAVIOR
            && (parentStruct == null || parentStruct.kind != IrModule.Declaration.Struct.Kind.ACTOR)
        ) {
            val label = Label(module.source, function.sourceLocation, "Behavior function must be within actor")
            val diagnostic = Diagnostic(
                Diagnostic.Severity.ERROR,
                "Behavior functions must be within actors",
                    mutableListOf(label)
                )
                diagnostics.add(diagnostic)
            }
    }

}