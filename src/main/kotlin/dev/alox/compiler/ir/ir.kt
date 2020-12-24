package dev.alox.compiler.ir

import dev.alox.compiler.ast.AstModule
import dev.alox.compiler.ast.Path
import dev.alox.compiler.report.SourceLocation

/**
 * IR representation of a module of code
 */
data class IrModule(val path: Path, val name: String, val declarations: List<Declaration>, val source: String) {
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
            val declarations: List<Declaration>,
            val sourceLocation: SourceLocation
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
            val returnType: DeclarationRef,
            val sourceLocation: SourceLocation
        ) : Declaration(name) {
            enum class Kind {
                FUNCTION,
                BEHAVIOR,
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
                fun fromReference(
                    compiler: IrCompiler,
                    currentModule: IrModule,
                    declarationRef: DeclarationRef
                ): Type? {
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
                                val type = compiler.resolve(currentModule, declarationRef.arguments[0])
                                if (type is Type) Ref(type) else null
                            } else if (name == "Array" && declarationRef.arguments.size == 1) {
                                val type = compiler.resolve(currentModule, declarationRef.arguments[0])
                                if (type is Type) Array(type) else null
                            } else {
                                null
                            }
                        }
                    }
                }

                fun fromDeclaration(declaration: Declaration): Type {
                    return when (declaration) {
                        is Declaration.Function -> Function(declaration, mapOf())
                        is Declaration.Struct -> Struct(declaration, mapOf())
                        is Type -> declaration
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
            "${if (path.path.isEmpty()) "" else "$path::"}$name" +
                    if (arguments.isEmpty()) "" else "[${arguments.joinToString(", ")}]"
    }

    /**
     * Block of instructions
     */
    data class Block(val id: Int, val instructions: MutableList<Instruction> = mutableListOf())

    /**
     * An SSA instruction
     */
    sealed class Instruction {

        data class Load(val ptr: Instruction) : Instruction()
        data class Store(val ptr: Instruction, val value: Instruction) : Instruction()
        data class Alloca(val name: String, val declarationRef: DeclarationRef) : Instruction()
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
        data class New(val struct: DeclarationRef) : Instruction()
        data class MethodCall(val aggregate: Instruction, val methodName: String, val arguments: List<Instruction>) :
            Instruction() {

            /**
             * Get the method declaration given the context
             */
            fun getMethod(
                compiler: IrCompiler,
                currentModule: IrModule,
                currentFunction: Declaration.Function,
                parentStruct: Declaration.Struct? = null
            ): Declaration.Function? {
                var outerType = aggregate.getType(compiler, currentModule, currentFunction, parentStruct)
                while(outerType is Declaration.Type.Ref) {
                    outerType = outerType.inner
                }
                val structType = outerType
                    ?.let { if (it is Declaration.Type.Struct) it else null }
                return structType
                    ?.declaration?.declarations
                    ?.firstOrNull { it.name == this.methodName }
                    ?.let { if (it is Declaration.Function) it else null }
            }
        }

        sealed class BinaryOperator(open val lhs: Instruction, open val rhs: Instruction) : Instruction() {
            data class Add(override val lhs: Instruction, override val rhs: Instruction) : BinaryOperator(lhs, rhs)
            data class Sub(override val lhs: Instruction, override val rhs: Instruction) : BinaryOperator(lhs, rhs)
            data class Mul(override val lhs: Instruction, override val rhs: Instruction) : BinaryOperator(lhs, rhs)
            data class Div(override val lhs: Instruction, override val rhs: Instruction) : BinaryOperator(lhs, rhs)
        }

        object This : Instruction()

        /**
         * Get the Type of this instruction based on the context
         */
        fun getType(
            compiler: IrCompiler,
            currentModule: IrModule,
            currentFunction: Declaration.Function,
            parentStruct: Declaration.Struct? = null
        ): Declaration.Type? {
            return when (this) {
                is Load -> this.ptr.getType(compiler, currentModule, currentFunction, parentStruct)
                is Store -> Declaration.Type.Primitive.Void
                is Alloca -> compiler.resolve(currentModule, this.declarationRef)
                    ?.let { if (it is Declaration.Type) it else null }
                is BooleanLiteral -> Declaration.Type.Primitive.Bool
                is IntegerLiteral -> Declaration.Type.Primitive.Int32
                is FloatLiteral -> Declaration.Type.Primitive.Float32
                is GetParameter -> {
                    currentFunction.arguments.firstOrNull { it.name == this.name }
                        ?.let { compiler.resolve(currentModule, it.declarationRef) }
                        ?.let { if (it is Declaration.Type) it else null }
                }
                is DeclarationReference -> compiler.resolve(currentModule, this.declarationRef)
                    ?.let { if (it is Declaration.Type) it else null }
                is FunctionCall -> {
                    this.function.getType(compiler, currentModule, currentFunction, parentStruct)
                        ?.let { if (it is Declaration.Type.Function) it else null }
                        ?.let { compiler.resolve(currentModule, it.declaration.returnType) }
                        ?.let { if (it is Declaration.Type) it else null }
                }
                is GetField -> {
                    this.aggregate.getType(compiler, currentModule, currentFunction, parentStruct)
                        ?.let { if (it is Declaration.Type.Struct) it else null }
                        ?.let { it.declaration.fields.firstOrNull { it.name == this.field }?.declarationRef }
                        ?.let { compiler.resolve(currentModule, it) }
                        ?.let { if (it is Declaration.Type) it else null }
                }
                is Return -> Declaration.Type.Primitive.Void
                is Jump -> Declaration.Type.Primitive.Void
                is Branch -> Declaration.Type.Primitive.Void
                is Dereference -> TODO()
                is AddressOf -> TODO()
                is New -> TODO()
                is MethodCall -> TODO()
                is BinaryOperator.Add -> Declaration.Type.Primitive.Int32
                is BinaryOperator.Sub -> Declaration.Type.Primitive.Int32
                is BinaryOperator.Mul -> Declaration.Type.Primitive.Int32
                is BinaryOperator.Div -> Declaration.Type.Primitive.Int32
                This -> parentStruct?.let {
                    Declaration.Type.Ref(
                        Declaration.Type.Struct(
                            it,
                            mapOf()/*TODO generics*/
                        )
                    )
                }
            }
        }
    }
}
