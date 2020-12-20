package dev.alox.compiler.backend

import dev.alox.compiler.ir.IrCompiler
import dev.alox.compiler.ir.IrModule
import org.bytedeco.javacpp.PointerPointer
import org.bytedeco.llvm.LLVM.*
import org.bytedeco.llvm.global.LLVM.*

/**
 * Translates an IR Module to LLVM IR
 */
class LLVMBackend(private val compiler: IrCompiler, private val irModule: IrModule) {

    private val context: LLVMContextRef
    private val builder: LLVMBuilderRef
    private val llvmModule: LLVMModuleRef
    private val typeCache = mutableMapOf<IrModule.Declaration.Type, LLVMTypeRef>()
    private val declarationCache = mutableMapOf<IrModule.Declaration, Pair<LLVMValueRef?, LLVMTypeRef>>()

    init {
        LLVMInitializeX86TargetInfo()
        LLVMInitializeX86Target()
        LLVMInitializeX86TargetMC()
        LLVMInitializeX86AsmPrinter()
        context = LLVMContextCreate()
        llvmModule = LLVMModuleCreateWithNameInContext(irModule.name, context)
        builder = LLVMCreateBuilderInContext(context)
    }

    fun close() {
        LLVMDisposeModule(llvmModule)
        LLVMDisposeBuilder(builder)
        LLVMContextDispose(context)
    }

    private fun convertType(type: IrModule.Declaration.Type, parent: IrModule.Declaration.Type? = null): LLVMTypeRef? {
        return when (type) {
            is IrModule.Declaration.Type.Primitive.Void -> LLVMVoidTypeInContext(context)
            is IrModule.Declaration.Type.Primitive.IntT -> LLVMIntTypeInContext(context, type.bits)
            is IrModule.Declaration.Type.Primitive.FloatT -> {
                when (type.bits) {
                    16 -> LLVMHalfTypeInContext(context)
                    32 -> LLVMFloatTypeInContext(context)
                    64 -> LLVMDoubleTypeInContext(context)
                    128 -> LLVMFP128TypeInContext(context)
                    else -> null
                }
            }
            is IrModule.Declaration.Type.Primitive.Bool -> LLVMIntTypeInContext(context, 1)
            is IrModule.Declaration.Type.Ref -> {
                getType(type.inner)?.let { LLVMPointerType(it, 0) }
            }
            is IrModule.Declaration.Type.Function -> {
                val argTypes = type.declaration.arguments
                    .mapNotNull { it.declarationRef.toType() }
                    .toMutableList()
                if (parent != null) {
                    argTypes.add(0, parent)
                }

                val llvmArgTypes = PointerPointer<LLVMTypeRef>(
                    *argTypes
                        .map { getType(it) }
                        .toTypedArray()
                )

                val returnType = type.declaration.returnType.toType()
                    ?.let { getType(it) }

                LLVMFunctionType(returnType, llvmArgTypes, argTypes.size, 0)
            }
            is IrModule.Declaration.Type.Struct -> {
                val struct = type.declaration
                val llvmStruct = LLVMStructCreateNamed(context, struct.name)

                // convert the fields to LLVMTypeRefs by resolving them using the compiler and converting them
                // TODO: support type parameters
                val fieldTypes = struct.fields.mapNotNull { it.declarationRef.toType() }

                val fields = PointerPointer<LLVMTypeRef>(
                    *fieldTypes.map { getType(it) }.toTypedArray()
                )

                LLVMStructSetBody(llvmStruct, fields, fieldTypes.size, 0)
                llvmStruct
            }
            else -> {
                println("Failed to find LLVM Type for $type")
                null
            }
        }
    }

    /**
     * Convert from our IR Type to the LLVM IR Type and use the cache
     */
    fun getType(type: IrModule.Declaration.Type, parent: IrModule.Declaration.Type? = null): LLVMTypeRef? {
        var llvmType = typeCache[type]
        if (llvmType == null) {
            llvmType = convertType(type, parent)
            if (llvmType != null) {
                typeCache[type] = llvmType
            }
        }
        return llvmType
    }

    fun getType(declarationRef: IrModule.DeclarationRef, parent: IrModule.Declaration.Type? = null): LLVMTypeRef? {
        return compiler.resolve(irModule, declarationRef)?.let {
            if (it is IrModule.Declaration.Type) {
                getType(it, parent)
            } else {
                null
            }
        }
    }

    fun process() {
        irModule.declarations.forEach {
            processDeclarationHeader(it)
        }
        irModule.declarations.forEach {
            processDeclaration(it)
        }
    }

    /**
     * Generate anything needed to refer to a declaration out of order, like
     * functions calling later defined functions needing a declaration
     */
    private fun processDeclarationHeader(
        declaration: IrModule.Declaration,
        parentStruct: IrModule.Declaration.Struct? = null
    ) {
        when (declaration) {
            is IrModule.Declaration.Function -> {
                // Declares the function in the LLVM module without creating a body for it
                val functionType = IrModule.Declaration.Type.fromDeclaration(declaration)

                // the parent type is a pointer to the parent struct
                val parentType = parentStruct?.let {
                    IrModule.Declaration.Type.Ref(IrModule.Declaration.Type.Struct(it, mapOf()))
                }
                val llvmFunctionType = getType(functionType, parentType)!!

                // prepend the parent struct name to the function name
                val functionName =
                    if (parentStruct == null) declaration.name else parentStruct.name + "_" + declaration.name

                val llvmFunction = LLVMAddFunction(llvmModule, functionName, llvmFunctionType)
                declarationCache[declaration] = llvmFunction to llvmFunctionType
            }
            is IrModule.Declaration.Struct -> {
                val structType = IrModule.Declaration.Type.fromDeclaration(declaration)
                val llvmStructType = getType(structType)!!
                declarationCache[declaration] = null to llvmStructType
                declaration.declarations.forEach { processDeclarationHeader(it, declaration) }
            }
            else -> {
            }
        }
    }

    private fun processDeclaration(
        declaration: IrModule.Declaration,
        parentStruct: IrModule.Declaration.Struct? = null
    ) {
        when (declaration) {
            is IrModule.Declaration.Function -> {
                val (llvmValueRef, llvmTypeRef) = declarationCache[declaration]!!
                LLVMSetFunctionCallConv(llvmValueRef, LLVMCCallConv);
                processFunction(declaration, llvmValueRef!!, llvmTypeRef, parentStruct)
            }
            is IrModule.Declaration.Struct -> {
                declaration.declarations.forEach { processDeclaration(it, declaration) }
            }
        }
    }

    private fun processFunction(
        function: IrModule.Declaration.Function,
        llvmFunction: LLVMValueRef,
        llvmFunctionType: LLVMTypeRef,
        parentStruct: IrModule.Declaration.Struct? = null
    ) {
        // generate llvm blocks for every ir block
        // we do this before generating the instructions because instructions can refer to
        // blocks, like jump and branch
        val blockMap = function.blocks.map { block ->
            block to LLVMAppendBasicBlockInContext(context, llvmFunction, "block${block.id}")
        }.toMap()
        val insMap = mutableMapOf<IrModule.Instruction, LLVMValueRef>()

        function.blocks.forEach { block ->
            LLVMPositionBuilderAtEnd(builder, blockMap[block])

            block.instructions.forEach { ins ->
                val llvmIns = when (ins) {
                    is IrModule.Instruction.IntegerLiteral ->
                        LLVMConstInt(getType(IrModule.Declaration.Type.Primitive.Int32), ins.value, 0)
                    is IrModule.Instruction.BooleanLiteral ->
                        LLVMConstInt(getType(IrModule.Declaration.Type.Primitive.Bool), if (ins.value) 1 else 0, 0)
                    is IrModule.Instruction.Alloca -> {
                        val type = getType(ins.declarationRef)
                        LLVMBuildAlloca(builder, type, ins.name)
                    }
                    is IrModule.Instruction.Load -> {
                        val ptr = insMap[ins.ptr]
                        LLVMBuildLoad(builder, ptr, "")
                    }
                    is IrModule.Instruction.Store -> {
                        val ptr = insMap[ins.ptr]
                        val value = insMap[ins.value]
                        LLVMBuildStore(builder, value, ptr)
                    }
                    is IrModule.Instruction.FloatLiteral -> null
                    is IrModule.Instruction.GetParameter -> {
                        val arg = function.arguments.withIndex().first { it.value.name == ins.name }
                        var index = arg.index
                        // if there's a parent struct, the function's first argument is the struct
                        if (parentStruct != null) {
                            index++
                        }
                        LLVMGetParam(llvmFunction, index)
                    }
                    is IrModule.Instruction.DeclarationReference -> {
                        // todo: support declarations outside module
                        ins.declarationRef.resolve()
                            ?.let { if (it is IrModule.Declaration.Type.Struct) it.declaration else it }
                            ?.let { if (it is IrModule.Declaration.Type.Function) it.declaration else it }
                            ?.let { declarationCache[it]?.first }
                    }
                    is IrModule.Instruction.FunctionCall -> {
                        val func = insMap[ins.function]
                        val args = ins.arguments.map { insMap[it] }
                        val argPtr = PointerPointer<LLVMValueRef>(*args.toTypedArray())
                        LLVMBuildCall(builder, func, argPtr, args.size, "")
                    }
                    is IrModule.Instruction.GetField -> {
                        // get the inner aggregate type
                        val aggregate = insMap[ins.aggregate]
                        val aggregateType = ins.aggregate.getType(function, parentStruct)
                        var innerType = aggregateType
                        while (innerType is IrModule.Declaration.Type.Ref) {
                            innerType = innerType.inner
                        }

                        // get the index of the field within the struct
                        val index = if (innerType is IrModule.Declaration.Type.Struct) {
                            innerType.declaration.fields.withIndex()
                                .firstOrNull { it.value.name == ins.field }?.index ?: -2
                        } else -1

                        if (aggregateType is IrModule.Declaration.Type.Ref) {
                            // build the pointer and load the value
                            val gep = LLVMBuildStructGEP2(builder, getType(innerType!!), aggregate, index, "")
                            LLVMBuildLoad(builder, gep, "")
                        } else {
                            // extract the value directly
                            LLVMBuildExtractValue(builder, aggregate, index, "")
                        }
                    }
                    is IrModule.Instruction.Return -> {
                        val value = insMap[ins.value]
                        LLVMBuildRet(builder, value)
                    }
                    is IrModule.Instruction.Jump -> {
                        LLVMBuildBr(builder, blockMap[ins.block])
                    }
                    is IrModule.Instruction.Branch -> {
                        val cond = insMap[ins.condition]
                        val trueBlock = blockMap[ins.trueBlock]
                        val falseBlock = blockMap[ins.falseBlock]
                        LLVMBuildCondBr(builder, cond, trueBlock, falseBlock)
                    }
                    is IrModule.Instruction.Dereference -> null
                    is IrModule.Instruction.AddressOf -> null
                    is IrModule.Instruction.New -> null
                    is IrModule.Instruction.MethodCall -> {
                        // get the llvm function for this method
                        val method: IrModule.Declaration.Function =
                            ins.getMethod(compiler, irModule, function, parentStruct)!!
                        val func = declarationCache[method]?.first

                        // build the args and append the struct to the beginning
                        val args = ins.arguments.map { insMap[it] }.toMutableList()
                        val aggregate = insMap[ins.aggregate]
                        args.add(0, aggregate)
                        val argPtr = PointerPointer<LLVMValueRef>(*args.toTypedArray())

                        LLVMBuildCall(builder, func, argPtr, args.size, "")
                    }
                    is IrModule.Instruction.BinaryOperator.Add -> null
                    is IrModule.Instruction.BinaryOperator.Sub -> null
                    is IrModule.Instruction.BinaryOperator.Mul -> null
                    is IrModule.Instruction.BinaryOperator.Div -> null
                    IrModule.Instruction.This -> LLVMGetParam(llvmFunction, 0)
                }
                if (llvmIns != null) {
                    insMap[ins] = llvmIns
                }
            }
        }
    }

    fun dump() {
        LLVMDumpModule(llvmModule)
    }

    // useful extension functions

    private fun IrModule.Instruction.getType(
        currentFunction: IrModule.Declaration.Function,
        parent: IrModule.Declaration.Struct? = null
    ) = getType(compiler, irModule, currentFunction, parent)

    private fun IrModule.DeclarationRef.resolve(): IrModule.Declaration? = compiler.resolve(irModule, this)

    private fun IrModule.DeclarationRef.toType(): IrModule.Declaration.Type? =
        compiler.resolve(irModule, this)
            ?.let { if (it is IrModule.Declaration.Type) it else null }
}
