package dev.alox.compiler.ir

import dev.alox.compiler.ast.AstModule
import dev.alox.compiler.ast.Path

/**
 * IR representation of a module of code
 */
data class IrModule(val path: Path, val name: String, val declarations: List<Declaration>) {
    sealed class Declaration(open val name: String) {

        /**
         * Resolve this Declaration using the Declaration Reference.
         * This is used for filling in Struct / Function generics.
         */
        open fun applyFrom(declarationRef: DeclarationRef): Declaration = this

        /**
         * A struct or actor
         */
        data class Struct(
            override val name: String,
            val kind: Kind,
            val typeParameters: List<String>,
            val fields: List<Field>,
            val declarations: List<Declaration>
        ) : Declaration(name) {
            enum class Kind {
                STRUCT,
                ACTOR
            }

            data class Field(val name: String, val declarationRef: DeclarationRef)

            /**
             * Apply a Declaration Reference's type arguments to this struct, generating a new Struct Type.
             */
            override fun applyFrom(declarationRef: DeclarationRef): Type.Struct =
                Type.Struct(
                    this,
                    declarationRef.arguments.mapIndexed { index, arg -> typeParameters[index] to arg }.toMap()
                )
        }

        /**
         * A function / behavior / kernel
         */
        data class Function(
            override val name: String,
            val kind: Kind,
            val typeParameters: List<String>,
            val arguments: List<Argument>,
            val blocks: List<Block>,
            val returnType: DeclarationRef
        ) : Declaration(name) {
            enum class Kind {
                FUNCTION,
                BEHAVIOUR,
                KERNEL;

                fun toAst(): AstModule.Declaration.Function.Kind =
                    AstModule.Declaration.Function.Kind.valueOf(this.name)
            }

            data class Argument(val name: String, val declarationRef: DeclarationRef)

            /**
             * Apply a Declaration Reference's type arguments to this struct, generating a new Struct Type.
             */
            override fun applyFrom(declarationRef: DeclarationRef): Type.Function =
                Type.Function(
                    this,
                    declarationRef.arguments.mapIndexed { index, arg -> typeParameters[index] to arg }.toMap()
                )
        }

        sealed class Type(override val name: String) : Declaration(name) {
            data class Unresolved(override val name: String) : Type(name)

            data class Struct(
                val declaration: Declaration.Struct,
                val typeParameters: Map<String, DeclarationRef>
            ) : Type(declaration.name)

            data class Function(
                val declaration: Declaration.Function,
                val typeParameters: Map<String, DeclarationRef>
            ) : Type(declaration.name)

            sealed class Primitive(override val name: String) : Type(name) {
                open class IntT(val bits: Int) : Primitive("Int$bits") {
                    override fun equals(other: Any?): Boolean {
                        return other is IntT && other.bits == bits
                    }
                }

                open class FloatT(val bits: Int) : Primitive("Float$bits") {
                    override fun equals(other: Any?): Boolean {
                        return other is IntT && other.bits == bits
                    }
                }

                object Int8 : IntT(8)
                object Int16 : IntT(16)
                object Int32 : IntT(32)
                object Int64 : IntT(64)
                object Int128 : IntT(128)

                object Float8 : FloatT(8)
                object Float16 : FloatT(16)
                object Float32 : FloatT(32)
                object Float64 : FloatT(64)
                object Float128 : FloatT(128)

                object Bool : Primitive("Bool")
                object Void : Primitive("Void")
                object NoReturn : Primitive("NoReturn")
            }

            data class Ref(val inner: Type) : Type("Ref")
            data class Array(val inner: Type) : Type("Array")

            companion object {
                /**
                 * Go over primitive and builtin types and find one that matches, resolving type parameters that need it
                 */
                fun fromReference(compiler: IrCompiler, declarationRef: DeclarationRef): Type? {
                    if (declarationRef.path.path.isNotEmpty()) return null
                    return when (val name = declarationRef.name) {
                        "Int8" -> Primitive.Int8
                        "Int16" -> Primitive.Int16
                        "Int32" -> Primitive.Int32
                        "Int64" -> Primitive.Int64
                        "Int128" -> Primitive.Int128
                        "Float8" -> Primitive.Float8
                        "Float16" -> Primitive.Float16
                        "Float32" -> Primitive.Float32
                        "Float64" -> Primitive.Float64
                        "Float128" -> Primitive.Float128
                        "Bool" -> Primitive.Bool
                        "Void" -> Primitive.Void
                        "NoReturn" -> Primitive.NoReturn
                        else -> {
                            if (name.startsWith("Int")) {
                                name.substring("Int".length).toIntOrNull()?.let { Primitive.IntT(it) }
                            } else if (name.startsWith("Float")) {
                                name.substring("Float".length).toIntOrNull()?.let { Primitive.FloatT(it) }
                            } else if (name == "Ref" && declarationRef.arguments.size == 1) {
                                val type = compiler.resolve(declarationRef.arguments[0])
                                if (type is Type) Ref(type) else null
                            } else if (name == "Array" && declarationRef.arguments.size == 1) {
                                val type = compiler.resolve(declarationRef.arguments[0])
                                if (type is Type) Array(type) else null
                            } else {
                                null
                            }
                        }
                    }
                }
            }
        }
    }

    /**
     * A reference to a declaration or type
     */
    data class DeclarationRef(val path: Path, val name: String, val arguments: List<DeclarationRef> = listOf()) {
        // path::Name[arg, arg]
        override fun toString(): String =
            "$path$name${if (arguments.isEmpty()) "" else "[${arguments.joinToString(", ")}]"}"
    }

    /**
     * Block of instructions
     */
    data class Block(val id: Int, val instructions: MutableList<Instruction> = mutableListOf())

    sealed class Instruction {
        data class Unreachable(val reason: String) : Instruction()
        data class Load(val ptr: Instruction) : Instruction()
        data class Store(val ptr: Instruction, val value: Instruction) : Instruction()
        data class Allocate(val name: String, val declarationRef: DeclarationRef) : Instruction()
        data class BooleanLiteral(val value: Boolean) : Instruction()
        data class IntegerLiteral(val value: Long) : Instruction()
        data class FloatLiteral(val value: Double) : Instruction()
        data class GetParameter(val name: String) : Instruction()
        data class DeclarationReference(val declarationRef: DeclarationRef) : Instruction()
        data class FunctionCall(val function: Instruction, val arguments: List<Instruction>) : Instruction()
        data class GetField(val aggregate: Instruction, val field: String) : Instruction()
        data class Return(val value: Instruction) : Instruction()
        data class Jump(val block: Block) : Instruction()
        data class Branch(val condition: Instruction, val trueBlock: Block, val falseBlock: Block) : Instruction()
        data class Dereference(val ptr: Instruction) : Instruction()
        data class AddressOf(val value: Instruction) : Instruction()
    }
}
