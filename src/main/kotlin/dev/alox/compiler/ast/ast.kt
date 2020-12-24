package dev.alox.compiler.ast

import dev.alox.compiler.ir.IrModule
import dev.alox.compiler.report.SourceLocation

data class Path(val path: List<String> = listOf()) {
    fun append(name: String) = Path(path.toMutableList().apply { add(name) })

    override fun toString(): String = path.joinToString("::")

    companion object {
        val empty = Path(listOf())
    }
}

/**
 * AST representation of a module containing code
 */
data class AstModule(val path: Path, val name: String, val declarations: List<Declaration>, val source: String) {

    /**
     * A reference to a Type that is later resolved to the real type
     */
    data class TypeName(val path: Path, val name: String, val arguments: List<TypeName>) {
        fun toIr(): IrModule.DeclarationRef = IrModule.DeclarationRef(path, name, arguments.map { it.toIr() })
    }

    sealed class Declaration(open val sourceLocation: SourceLocation) {
        /**
         * A struct or actor
         */
        data class Struct(
            val name: String,
            val kind: Kind,
            val typeParameters: List<String>,
            val fields: List<Field>,
            val declarations: List<Declaration>,
            override val sourceLocation: SourceLocation
        ) : Declaration(sourceLocation) {
            enum class Kind {
                STRUCT,
                ACTOR;

                fun toIr(): IrModule.Declaration.Struct.Kind = IrModule.Declaration.Struct.Kind.valueOf(this.name)
            }

            data class Field(val name: String, val typeName: TypeName)
        }

        /**
         * A function / behavior / kernel
         */
        class Function(
            val name: String,
            val kind: Kind,
            val typeParameters: List<String>,
            val arguments: List<Argument>,
            val statements: List<Statement>,
            val returnType: TypeName,
            override val sourceLocation: SourceLocation
        ) : Declaration(sourceLocation) {
            enum class Kind {
                FUNCTION,
                BEHAVIOR,
                KERNEL;

                fun toIr(): IrModule.Declaration.Function.Kind = IrModule.Declaration.Function.Kind.valueOf(this.name)

                companion object {
                    fun from(value: String): Kind = when (value) {
                        "fun" -> FUNCTION
                        "behave" -> BEHAVIOR
                        "kernel" -> KERNEL
                        else -> throw IllegalArgumentException("$value is not a Function Kind")
                    }
                }
            }

            data class Argument(val name: String, val typeName: TypeName)
        }
    }

    sealed class Statement {
        data class VariableDeclaration(val name: String, val type: TypeName) : Statement()
        data class Assignment(val aggregate: Expression, val value: Expression) : Statement()
        data class VariableDefinition(val name: String, val type: TypeName, val value: Expression) : Statement()
        data class FunctionCall(val function: Expression, val arguments: List<Expression>) : Statement()
        data class IfStatement(val condition: Expression, val block: List<Statement>, val elseif: IfStatement?) :
            Statement()

        data class Return(val value: Expression) : Statement()
        data class MethodCall(val aggregate: Expression, val methodName: String, val arguments: List<Expression>) :
            Statement()
    }

    sealed class Expression {
        data class BooleanLiteral(val value: Boolean) : Expression() {
            fun toIr(): IrModule.Instruction.BooleanLiteral = IrModule.Instruction.BooleanLiteral(value)
        }

        data class IntegerLiteral(val value: Long) : Expression() {
            fun toIr(): IrModule.Instruction.IntegerLiteral = IrModule.Instruction.IntegerLiteral(value)
        }

        data class FloatLiteral(val value: Double) : Expression() {
            fun toIr(): IrModule.Instruction.FloatLiteral = IrModule.Instruction.FloatLiteral(value)
        }

        data class BinaryOperator(val kind: Kind, val lhs: Expression, val rhs: Expression) : Expression() {
            enum class Kind(val op: String) {
                ADD("+"),
                SUBTRACT("-"),
                MULTIPLY("*"),
                DIVIDE("/"),
            }
        }

        data class VariableReference(val path: Path, val name: String) : Expression()
        data class FunctionCall(val function: Expression, val arguments: List<Expression>) : Expression()
        data class GetField(val struct: Expression, val field: String) : Expression()
        data class New(val struct: TypeName) : Expression()
        data class AddressOf(val value: Expression) : Expression()
        data class MethodCall(val aggregate: Expression, val methodName: String, val arguments: List<Expression>) :
            Expression()

        object This : Expression() {
            override fun toString(): String = "this"
        }
    }

}
