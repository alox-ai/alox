package dev.alox.compiler.backend

import dev.alox.compiler.ast.Path
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

    private fun convertType(type: IrModule.Declaration.Type): LLVMTypeRef? {
        return when (type) {
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
            is IrModule.Declaration.Type.Function -> {
                val argTypes = type.declaration.arguments
                    .map { arg -> compiler.resolve(irModule, arg.declarationRef) }
                    .filterIsInstance(IrModule.Declaration.Type::class.java)

                val llvmArgTypes = PointerPointer<LLVMTypeRef>(
                    *argTypes
                        .map { getType(it) }
                        .toTypedArray()
                )

                val returnType = compiler.resolve(irModule, type.declaration.returnType)
                    ?.takeIf { it is IrModule.Declaration.Type }
                    ?.let { it as IrModule.Declaration.Type }
                    ?.let { getType(it) }

                LLVMFunctionType(returnType, llvmArgTypes, type.declaration.arguments.size, 0)
            }
            is IrModule.Declaration.Type.Struct -> {
                val struct = type.declaration
                val llvmStruct = LLVMStructCreateNamed(context, struct.name)

                // convert the fields to LLVMTypeRefs by resolving them using the compiler and converting them
                // TODO: support type parameters
                val fieldTypes = struct.fields.map { compiler.resolve(irModule, it.declarationRef) }
                    .filterIsInstance(IrModule.Declaration.Type::class.java)

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
    fun getType(type: IrModule.Declaration.Type): LLVMTypeRef? {
        var llvmType = typeCache[type]
        if (llvmType == null) {
            llvmType = convertType(type)
            if (llvmType != null) {
                typeCache[type] = llvmType
            }
        }
        return llvmType
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
    private fun processDeclarationHeader(declaration: IrModule.Declaration) {
        when (declaration) {
            is IrModule.Declaration.Function -> {
                // Declares the function in the LLVM module without creating a body for it
                val functionType = IrModule.Declaration.Type.fromDeclaration(declaration)
                val llvmFunctionType = getType(functionType)!!
                val llvmFunction = LLVMAddFunction(llvmModule, declaration.name, llvmFunctionType)
                declarationCache[declaration] = llvmFunction to llvmFunctionType
            }
            is IrModule.Declaration.Struct -> {
                val structType = IrModule.Declaration.Type.fromDeclaration(declaration)
                val llvmStructType = getType(structType)!!
                declarationCache[declaration] = null to llvmStructType
            }
            else -> {
            }
        }
    }

    private fun processDeclaration(declaration: IrModule.Declaration) {
        when (declaration) {
            is IrModule.Declaration.Function -> {
                val (llvmValueRef, llvmTypeRef) = declarationCache[declaration]!!
                LLVMSetFunctionCallConv(llvmValueRef, LLVMCCallConv);
                processFunction(declaration, llvmValueRef!!, llvmTypeRef)
            }
        }
    }

    private fun processFunction(
        function: IrModule.Declaration.Function,
        llvmFunction: LLVMValueRef,
        llvmFunctionType: LLVMTypeRef
    ) {

    }

    fun dump() {
        LLVMDumpModule(llvmModule)
    }
}
