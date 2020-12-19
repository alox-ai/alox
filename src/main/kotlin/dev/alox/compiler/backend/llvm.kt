package dev.alox.compiler.backend

import dev.alox.compiler.ir.IrCompiler
import dev.alox.compiler.ir.IrModule
import org.bytedeco.javacpp.PointerPointer
import org.bytedeco.llvm.LLVM.*
import org.bytedeco.llvm.global.LLVM.*

/**
 * Translates an IR Module to LLVM IR
 */
class LLVMBackend(val compiler: IrCompiler, val irModule: IrModule) {

    val context: LLVMContextRef
    val builder: LLVMBuilderRef
    val llvmModule: LLVMModuleRef
    val typeCache = mutableMapOf<IrModule.Declaration.Type, LLVMTypeRef>()
    val declarationCache = mutableMapOf<IrModule.DeclarationRef, Pair<LLVMValueRef, LLVMTypeRef>>()

    init {
        LLVMInitializeX86TargetInfo()
        LLVMInitializeX86Target()
        LLVMInitializeX86TargetMC()
        LLVMInitializeX86AsmPrinter()
        context = LLVMContextCreate()
        llvmModule = LLVMModuleCreateWithNameInContext(irModule.name, context)
        builder = LLVMCreateBuilderInContext(context)
    }

    fun close(){
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
            is IrModule.Declaration.Type.Struct -> {
                val struct = type.declaration
                val llvmStruct = LLVMStructCreateNamed(context, struct.name)

                // convert the fields to LLVMTypeRefs by resolving them using the compiler and converting them
                // TODO: support type parameters
                val fields = PointerPointer<LLVMTypeRef>(
                    *struct.fields.map { compiler.resolve(it.declarationRef) }
                        .filterIsInstance(IrModule.Declaration.Type::class.java)
                        .map { getType(it) }
                        .toTypedArray()
                )

                LLVMStructSetBody(llvmStruct, fields, 0, 0)
                llvmStruct
            }
            else -> null
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

}
