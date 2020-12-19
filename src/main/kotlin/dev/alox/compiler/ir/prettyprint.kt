package dev.alox.compiler.ir

/**
 * Pretty Printer for IR Modules
 */
class PrettyPrinter(module: IrModule) {

    private var indent = 0

    private fun prefix(): String = "  ".repeat(indent)

    private fun p(s: String = "") = println("${prefix()}$s")

    init {
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
                                    is IrModule.Instruction.DeclarationReference -> "%${it.declarationRef}"
                                    is IrModule.Instruction.GetField -> "getfield %${insMap[it.aggregate]} \"${it.field}\""
                                    is IrModule.Instruction.Return -> "ret %${insMap[it.value]}"
                                    else -> "$it"
                                }
                            }"
                        )
                    }
                    indent--
                }
            }
            is IrModule.Declaration.Type -> {
                print("type $declaration")
            }
        }
    }

}
