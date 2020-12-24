package dev.alox.compiler.ir

/**
 * Pretty Printer for IR Modules
 */
class PrettyPrinter(module: IrModule) {

    private var indent = 0

    private fun prefix(): String = "  ".repeat(indent)

    private fun p(s: String = "") = println("${prefix()}$s")

    init {
        p("; Alox Module ${module.name}")
        module.declarations.forEach { prettyPrint(it) }
    }

    private fun prettyPrint(declaration: IrModule.Declaration) {
        when (declaration) {
            is IrModule.Declaration.Struct -> {
                declaration.apply {
                    p("${kind.name.toLowerCase()} $name[${typeParameters.joinToString()}]")
                }
                indent++
                declaration.fields.forEach {
                    p("let ${it.name} : ${it.declarationRef}")
                }
                p()
                declaration.declarations.forEach {
                    prettyPrint(it)
                }
                indent--
            }
            is IrModule.Declaration.Function -> {
                declaration.apply {
                    p("${kind.name.toLowerCase()} $name (${arguments.joinToString { "%${it.name}: ${it.declarationRef}" }}): $returnType")
                }
                indent++
                declaration.blocks.forEach { block ->
                    p("block#${block.id}:")
                    indent++
                    val insMap = mutableMapOf<IrModule.Instruction, Int>()
                    block.instructions.forEach {
                        val id = insMap.values.size
                        insMap[it] = id
                        p(
                            "%$id = ${
                                when (it) {
                                    is IrModule.Instruction.BooleanLiteral -> "${it.value}"
                                    is IrModule.Instruction.IntegerLiteral -> "$${it.value}"
                                    is IrModule.Instruction.FloatLiteral -> "$${it.value}"
                                    is IrModule.Instruction.This -> "this"
                                    is IrModule.Instruction.GetParameter -> "getparam %${it.name}"
                                    is IrModule.Instruction.DeclarationReference -> "%${it.declarationRef}"
                                    is IrModule.Instruction.GetField -> "getfield %${insMap[it.aggregate]} \"${it.field}\""
                                    is IrModule.Instruction.Return -> "ret %${insMap[it.value]}"
                                    is IrModule.Instruction.Alloca -> "alloca ${it.declarationRef}"
                                    is IrModule.Instruction.Store -> "store %${insMap[it.value]} in %${insMap[it.ptr]}"
                                    is IrModule.Instruction.Load -> "load %${insMap[it.ptr]}"
                                    is IrModule.Instruction.FunctionCall -> "call %${insMap[it.function]}(${it.arguments.joinToString { "%${insMap[it]}" }})"
                                    is IrModule.Instruction.MethodCall -> "callMethod %${insMap[it.aggregate]}.${it.methodName}(${it.arguments.joinToString { "%${insMap[it]}" }})"
                                    is IrModule.Instruction.AddressOf -> "&%${insMap[it.value]}"
                                    is IrModule.Instruction.Branch -> "branch %${insMap[it.condition]} block#${it.trueBlock.id} block#${it.falseBlock.id}"
                                    else -> "$it"
                                }
                            }"
                        )
                    }
                    indent--
                }
                indent--
            }
            is IrModule.Declaration.Type -> {
                print("type $declaration")
            }
        }
    }

}
