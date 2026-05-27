use crate::compiler::Backend;

#[cfg(feature = "cranelift")]
pub mod cranelift;
#[cfg(feature = "llvm")]
pub mod llvm;

pub mod tree;

pub fn has_llvm() -> bool {
    #[cfg(feature = "llvm")]
    {
        true
    }
    #[cfg(not(feature = "llvm"))]
    {
        false
    }
}

pub fn has_cranelift() -> bool {
    #[cfg(feature = "cranelift")]
    {
        true
    }
    #[cfg(not(feature = "cranelift"))]
    {
        false
    }
}
