package dev.alox.compiler.ir

import java.util.*

import dev.alox.compiler.ir.IrModule.*

/**
 * Compiler structure that handles running the pipeline concurrently
 */
class IrCompiler {

    private val modules: MutableList<IrModule> = Collections.synchronizedList(mutableListOf())

    fun addModule(irModule: IrModule) {
        modules.add(irModule)
    }

    /**
     * Resolve a declaration using a reference. This will fill in type parameters for defined and builtin types.
     */
    fun resolve(currentModule: IrModule, declarationRef: DeclarationRef): Declaration? {
        return if (declarationRef.path.path.isEmpty()) {
            // ref doesn't contain a path so it's probably looking for a type in the current module or a builtin type
            currentModule.declarations
                .firstOrNull { it.name == declarationRef.name }
                ?.applyFrom(declarationRef)
                ?: Declaration.Type.fromReference(this, currentModule, declarationRef)
        } else {
            // ref has a path so we know exactly where to find the declaration
            modules
                .firstOrNull { it.path == declarationRef.path }
                ?.declarations?.firstOrNull { it.name == declarationRef.name }
                ?.applyFrom(declarationRef)
        }
    }

}
