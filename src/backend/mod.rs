pub mod cranelift;
pub mod llvm;

pub enum Backend {
    CraneLift,
    LLVM,
    SPIRV,
    CUDA,
    OpenCL,
}

impl Backend {
    pub fn is_cpu(&self) -> bool {
        match self {
            Backend::CraneLift | Backend::LLVM => true,
            _ => false
        }
    }
}
