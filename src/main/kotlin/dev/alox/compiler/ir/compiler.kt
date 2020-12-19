package dev.alox.compiler.ir

import java.util.*

import dev.alox.compiler.ir.IrModule.*

/**
 * Compiler structure that handles running the pipeline concurrently
 */
class IrCompiler {

    private val modules: List<IrModule> = Collections.synchronizedList(mutableListOf())

    /**
     * Resolve a declaration using a reference. This will fill in type parameters for defined and builtin types.
     */
    fun resolve(declarationRef: DeclarationRef): Declaration? {
        return if (declarationRef.path.path.isEmpty()) {
            // ref doesn't contain a path so it's probably looking for a builtin type
            Declaration.Type.fromReference(this, declarationRef)
        } else {
            // ref has a path so we know exactly where to find the declaration
            modules
                .firstOrNull { it.path == declarationRef.path }
                ?.declarations?.firstOrNull { it.name == declarationRef.name }
                ?.applyFrom(declarationRef)
        }
    }

}
