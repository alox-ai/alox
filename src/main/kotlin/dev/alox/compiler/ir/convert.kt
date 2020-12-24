package dev.alox.compiler.ir

import dev.alox.compiler.ast.AstModule
import dev.alox.compiler.ast.Path
import kotlin.collections.ArrayDeque

/**
 * Handles translation from AST to IR
 */
class Translator(private val astModule: AstModule) {

    private val path: Path = astModule.path.append(astModule.name)

    /**
     * Generate an IR Module given the AST Module
     */
    fun generateModule(): IrModule {
        val declarations = astModule.declarations.map { generateDeclaration(it) }
        return IrModule(astModule.path, astModule.name, declarations, astModule.source)
    }

    /**
     * Generate an IR Declaration for an AST Declaration
     */
    fun generateDeclaration(astDeclaration: AstModule.Declaration): IrModule.Declaration {
        return when (astDeclaration) {
            is AstModule.Declaration.Function -> {
                val arguments =
                    astDeclaration.arguments.map { IrModule.Declaration.Function.Argument(it.name, it.typeName.toIr()) }
                val blockBuilder = BlockBuilder()
                val returnType = astDeclaration.returnType.toIr()
                val lvt = LocalVariableTable()

                astDeclaration.statements.forEach { statement ->
                    generateStatement(statement, blockBuilder, lvt, astDeclaration)
                }

                IrModule.Declaration.Function(
                    astDeclaration.name,
                    astDeclaration.kind.toIr(),
                    astDeclaration.typeParameters,
                    arguments,
                    blockBuilder.blocks,
                    returnType,
                    astDeclaration.sourceLocation
                )
            }
            is AstModule.Declaration.Struct -> {
                val fields =
                    astDeclaration.fields.map { IrModule.Declaration.Struct.Field(it.name, it.typeName.toIr()) }
                val declarations = astDeclaration.declarations.map { generateDeclaration(it) }
                IrModule.Declaration.Struct(
                    astDeclaration.name,
                    astDeclaration.kind.toIr(),
                    astDeclaration.typeParameters,
                    fields,
                    declarations,
                    astDeclaration.sourceLocation
                )
            }
        }
    }

    /**
     * Generate IR Instructions for an AST Statement
     */
    fun generateStatement(
        statement: AstModule.Statement,
        blockBuilder: BlockBuilder,
        lvt: LocalVariableTable,
        context: AstModule.Declaration.Function
    ) {
        when (statement) {
            is AstModule.Statement.VariableDeclaration -> {
                // allocate space for the variable
                val allocate = IrModule.Instruction.Alloca(statement.name, statement.type.toIr())
                blockBuilder.addInstruction(allocate)
                lvt[statement.name] = allocate
            }
            is AstModule.Statement.Assignment -> {
                // gen the expression instructions
                val aggregate = generateExpression(statement.aggregate, blockBuilder, lvt, context)
                val value = generateExpression(statement.value, blockBuilder, lvt, context)

                // store the value
                val store = IrModule.Instruction.Store(aggregate, value)
                blockBuilder.addInstruction(store)
            }
            is AstModule.Statement.VariableDefinition -> {
                // create the value to store in the variable
                val value = generateExpression(statement.value, blockBuilder, lvt, context)

                // allocate space for the variable
                val allocate = IrModule.Instruction.Alloca(statement.name, statement.type.toIr())
                blockBuilder.addInstruction(allocate)
                lvt[statement.name] = allocate

                // store the value in the variable
                val store = IrModule.Instruction.Store(allocate, value)
                blockBuilder.addInstruction(store)
            }
            is AstModule.Statement.FunctionCall -> {
                // gen the instructions for the function and the arguments
                val function = generateExpression(statement.function, blockBuilder, lvt, context)
                val arguments = statement.arguments.map { generateExpression(it, blockBuilder, lvt, context) }

                // call the function
                val call = IrModule.Instruction.FunctionCall(function, arguments)
                blockBuilder.addInstruction(call)
            }
            is AstModule.Statement.IfStatement -> {
                // gen condition
                val currentBlock = blockBuilder.currentBlock
                val condition = generateExpression(statement.condition, blockBuilder, lvt, context)

                // gen true block
                val trueBlock = blockBuilder.createBlock()
                statement.block.forEach {
                    generateStatement(it, blockBuilder, lvt, context)
                }

                val falseBlock = blockBuilder.createBlock()
                // gen else if block
                if (statement.elseif != null) {
                    generateStatement(statement.elseif, blockBuilder, lvt, context)
                }

                // gen branch instruction and add it to the original block
                val branch = IrModule.Instruction.Branch(condition, trueBlock, falseBlock)
                currentBlock.instructions.add(branch)
            }
            is AstModule.Statement.Return -> {
                val value = generateExpression(statement.value, blockBuilder, lvt, context)
                val ret = IrModule.Instruction.Return(value)
                blockBuilder.addInstruction(ret)
            }
            is AstModule.Statement.MethodCall -> {
                val aggregate = generateExpression(statement.aggregate, blockBuilder, lvt, context)
                val arguments = statement.arguments.map { generateExpression(it, blockBuilder, lvt, context) }

                val methodCall = IrModule.Instruction.MethodCall(aggregate, statement.methodName, arguments)
                blockBuilder.addInstruction(methodCall)
            }
        }
    }

    /**
     * Generate IR Instructions from an AST Expression
     */
    fun generateExpression(
        expression: AstModule.Expression,
        blockBuilder: BlockBuilder,
        lvt: LocalVariableTable,
        context: AstModule.Declaration.Function
    ): IrModule.Instruction {
        val instruction: IrModule.Instruction = when (expression) {
            is AstModule.Expression.BooleanLiteral -> expression.toIr()
            is AstModule.Expression.IntegerLiteral -> expression.toIr()
            is AstModule.Expression.FloatLiteral -> expression.toIr()
            is AstModule.Expression.FunctionCall -> {
                // gen the instructions for the function and the arguments
                val function = generateExpression(expression.function, blockBuilder, lvt, context)
                val arguments = expression.arguments.map { generateExpression(it, blockBuilder, lvt, context) }

                // call the function
                IrModule.Instruction.FunctionCall(function, arguments)
            }
            is AstModule.Expression.GetField -> {
                val struct = generateExpression(expression.struct, blockBuilder, lvt, context)
                IrModule.Instruction.GetField(struct, expression.field)
            }
            is AstModule.Expression.VariableReference -> {
                if (expression.path.path.isNotEmpty()) {
                    // this is a declaration to something in a module
                    val declarationId = IrModule.DeclarationRef(expression.path, expression.name)
                    IrModule.Instruction.DeclarationReference(declarationId)
                } else {
                    val localValue = lvt[expression.name]
                    if (localValue != null) {
                        // it's a local variable, load it
                        IrModule.Instruction.Load(localValue)
                    } else {
                        val arg = context.arguments.firstOrNull { it.name == expression.name }
                        if (arg != null) {
                            // it's an argument, load it
                            IrModule.Instruction.GetParameter(expression.name)
                        } else {
                            val declarationId = IrModule.DeclarationRef(path, expression.name)
                            IrModule.Instruction.DeclarationReference(declarationId)
                        }
                    }
                }
            }
            is AstModule.Expression.New -> IrModule.Instruction.New(expression.struct.toIr())
            is AstModule.Expression.BinaryOperator -> {
                val lhs = generateExpression(expression.lhs, blockBuilder, lvt, context)
                val rhs = generateExpression(expression.rhs, blockBuilder, lvt, context)
                when (expression.kind) {
                    AstModule.Expression.BinaryOperator.Kind.ADD -> {
                        IrModule.Instruction.BinaryOperator.Add(lhs, rhs)
                    }
                    AstModule.Expression.BinaryOperator.Kind.SUBTRACT -> {
                        IrModule.Instruction.BinaryOperator.Sub(lhs, rhs)
                    }
                    AstModule.Expression.BinaryOperator.Kind.MULTIPLY -> {
                        IrModule.Instruction.BinaryOperator.Mul(lhs, rhs)
                    }
                    AstModule.Expression.BinaryOperator.Kind.DIVIDE -> {
                        IrModule.Instruction.BinaryOperator.Div(lhs, rhs)
                    }
                }
            }
            is AstModule.Expression.AddressOf -> {
                val value = generateExpression(expression.value, blockBuilder, lvt, context)
                IrModule.Instruction.AddressOf(value)
            }
            is AstModule.Expression.MethodCall -> {
                val aggregate = generateExpression(expression.aggregate, blockBuilder, lvt, context)
                val arguments = expression.arguments.map { generateExpression(it, blockBuilder, lvt, context) }
                IrModule.Instruction.MethodCall(aggregate, expression.methodName, arguments)
            }
            is AstModule.Expression.This -> IrModule.Instruction.This
        }
        blockBuilder.addInstruction(instruction)
        return instruction
    }

}

/**
 * Handles the creation and management of blocks
 */
class BlockBuilder {
    val blocks = mutableListOf<IrModule.Block>()
    var currentBlock: IrModule.Block = IrModule.Block(0)

    init {
        blocks.add(currentBlock)
    }

    fun createBlock(): IrModule.Block {
        // don't create a new block if the current block has 0 instructions
        if (currentBlock.instructions.size > 0) {
            currentBlock = IrModule.Block(blocks.size)
            blocks.add(currentBlock)
        }
        return currentBlock
    }

    fun addInstruction(instruction: IrModule.Instruction) {
        currentBlock.instructions.add(instruction)
    }
}

@OptIn(ExperimentalStdlibApi::class)
class LocalVariableTable {
    private val table: ArrayDeque<MutableMap<String, IrModule.Instruction>> = ArrayDeque()

    init {
        pushDepth()
    }

    fun pushDepth() = table.addLast(mutableMapOf())

    fun popDepth() = table.removeLast()

    operator fun get(name: String): IrModule.Instruction? {
        table.reversed().forEach { map ->
            map[name]?.let { return it }
        }
        return null
    }

    operator fun set(name: String, ins: IrModule.Instruction) {
        table.lastOrNull()?.put(name, ins)
    }

}
