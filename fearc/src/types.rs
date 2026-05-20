use fear::{compiler::Backend, ir::*, types::*};

/// Opaque wrapper for the internal `Module` struct
#[repr(C)]
pub struct FearModule {
    __: [i8; 0],
}

/// Opaque wrapper for the internal `FunctionDef` struct, used to build function bodies.
#[repr(C)]
pub struct FearFunctionDef {
    __: [i8; 0],
}

pub type FearFuncId = u32;
pub type FearBlockId = u32;
pub type FearValueId = u32;

/// Supported types
#[repr(C)]
#[derive(Clone, Copy)]
pub enum FearType {
    FearVoid,
    FearBool,
    FearInt8,
    FearInt16,
    FearInt32,
    FearInt64,
    FearFloat32,
    FearFloat64,
    FearPointer,
}

/// Optimization levels mapping directly to compiler backend passes.
#[repr(C)]
#[derive(Clone, Copy)]
pub enum FearOptLevel {
    FearOptLevelNone,
    FearOptLevelDefault,
    FearOptLevelFull,
}

/// Calling conventions supported for function declarations and calls.
#[repr(C)]
#[derive(Clone, Copy)]
pub enum FearCallConv {
    FearCallConvC,
    FearCallConvSysV,
    FearCallConvMsAbi,
}

/// Linkage types specifying visibility and resolution rules for symbols.
#[repr(C)]
#[derive(Clone, Copy)]
pub enum FearLinkage {
    FearLinkageExternal,
    FearLinkageInternal,
    FearLinkageWeak,
}

/// Fear backend type
#[repr(C)]
#[derive(Clone, Copy)]
pub enum FearBackend {
    /// default backend
    FearBackendCranelift,
    /// not yet implemented
    FearBackendSelf,
    FearBackendLlvm,
}

/// Integer compare predicate
#[repr(C)]
#[derive(Clone, Copy)]
pub enum FearIntCmp {
    FearIntCmpEq,
    FearIntCmpNe,
    FearIntCmpLt,
    FearIntCmpLe,
    FearIntCmpGt,
    FearIntCmpGe,
}

/// Float compare predicate
#[repr(C)]
#[derive(Clone, Copy)]
pub enum FearFloatCmp {
    FearFloatCmpOrd,
    FearFloatCmpOrdEq,
    FearFloatCmpOrdNe,
    FearFloatCmpOrdLt,
    FearFloatCmpOrdLe,
    FearFloatCmpOrdGt,
    FearFloatCmpOrdGe,

    FearFloatCmpUno,
    FearFloatCmpUnoEq,
    FearFloatCmpUnoNe,
    FearFloatCmpUnoLt,
    FearFloatCmpUnoLe,
    FearFloatCmpUnoGt,
    FearFloatCmpUnoGe,
}

impl From<FearType> for Type {
    fn from(t: FearType) -> Self {
        match t {
            FearType::FearVoid => Type::Void,
            FearType::FearBool => Type::I1,
            FearType::FearInt8 => Type::I8,
            FearType::FearInt16 => Type::I16,
            FearType::FearInt32 => Type::I32,
            FearType::FearInt64 => Type::I64,
            FearType::FearFloat32 => Type::F32,
            FearType::FearFloat64 => Type::F64,
            FearType::FearPointer => Type::Ptr,
        }
    }
}

impl From<FearOptLevel> for OptLevel {
    fn from(t: FearOptLevel) -> Self {
        match t {
            FearOptLevel::FearOptLevelNone => OptLevel::None,
            FearOptLevel::FearOptLevelDefault => OptLevel::Default,
            FearOptLevel::FearOptLevelFull => OptLevel::Full,
        }
    }
}

impl From<FearCallConv> for CallingConvention {
    fn from(t: FearCallConv) -> Self {
        match t {
            FearCallConv::FearCallConvC => CallingConvention::C,
            FearCallConv::FearCallConvSysV => CallingConvention::SystemV,
            FearCallConv::FearCallConvMsAbi => CallingConvention::MicrosoftAbi,
        }
    }
}

impl From<FearLinkage> for Linkage {
    fn from(t: FearLinkage) -> Self {
        match t {
            FearLinkage::FearLinkageExternal => Linkage::External,
            FearLinkage::FearLinkageInternal => Linkage::Internal,
            FearLinkage::FearLinkageWeak => Linkage::Weak,
        }
    }
}

impl From<FearBackend> for Backend {
    fn from(t: FearBackend) -> Self {
        match t {
            FearBackend::FearBackendCranelift => Backend::Cranelift,
            FearBackend::FearBackendSelf => Backend::Fear,
            FearBackend::FearBackendLlvm => Backend::Llvm,
        }
    }
}

impl From<FearIntCmp> for IntCmp {
    fn from(t: FearIntCmp) -> Self {
        match t {
            FearIntCmp::FearIntCmpEq => IntCmp::Eq,
            FearIntCmp::FearIntCmpNe => IntCmp::Ne,
            FearIntCmp::FearIntCmpLt => IntCmp::Lt,
            FearIntCmp::FearIntCmpLe => IntCmp::Le,
            FearIntCmp::FearIntCmpGt => IntCmp::Gt,
            FearIntCmp::FearIntCmpGe => IntCmp::Ge,
        }
    }
}

impl From<FearFloatCmp> for FloatCmp {
    fn from(t: FearFloatCmp) -> Self {
        match t {
            FearFloatCmp::FearFloatCmpOrd => FloatCmp::Ord,
            FearFloatCmp::FearFloatCmpOrdEq => FloatCmp::OEq,
            FearFloatCmp::FearFloatCmpOrdNe => FloatCmp::ONe,
            FearFloatCmp::FearFloatCmpOrdLt => FloatCmp::OLt,
            FearFloatCmp::FearFloatCmpOrdLe => FloatCmp::OLe,
            FearFloatCmp::FearFloatCmpOrdGt => FloatCmp::OGt,
            FearFloatCmp::FearFloatCmpOrdGe => FloatCmp::OGe,

            FearFloatCmp::FearFloatCmpUno => FloatCmp::Uno,
            FearFloatCmp::FearFloatCmpUnoEq => FloatCmp::UEq,
            FearFloatCmp::FearFloatCmpUnoNe => FloatCmp::UNe,
            FearFloatCmp::FearFloatCmpUnoLt => FloatCmp::ULt,
            FearFloatCmp::FearFloatCmpUnoLe => FloatCmp::ULe,
            FearFloatCmp::FearFloatCmpUnoGt => FloatCmp::UGt,
            FearFloatCmp::FearFloatCmpUnoGe => FloatCmp::UGe,
        }
    }
}
