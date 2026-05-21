use clap::ValueEnum;
use enum_display::EnumDisplay;

#[repr(C)]
#[derive(Debug, EnumDisplay, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Type {
    #[default]
    #[display("void")]
    Void,
    #[display("bool")]
    I1,
    #[display("i8")]
    I8,
    #[display("i16")]
    I16,
    #[display("i32")]
    I32,
    #[display("i64")]
    I64,
    #[display("f32")]
    F32,
    #[display("f64")]
    F64,
    #[display("ptr")]
    Ptr,
}

impl Type {
    pub fn is_integer(&self) -> bool {
        matches!(self, Type::I8 | Type::I16 | Type::I32 | Type::I64)
    }

    pub fn is_float(&self) -> bool {
        matches!(self, Type::F32 | Type::F64)
    }

    pub fn is_void(&self) -> bool {
        matches!(self, Type::Void)
    }

    pub fn get_size(&self) -> usize {
        match self {
            Self::Void => 0,
            Self::I1 => 1,
            Self::I8 => 1,
            Self::I16 => 2,
            Self::I32 => 4,
            Self::I64 => 8,
            Self::F32 => 4,
            Self::F64 => 8,
            Self::Ptr => crate::target::HOST_DESC.pointer_size,
        }
    }
}

#[derive(Debug, Clone, Copy, ValueEnum)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum OptLevel {
    None,
    Default,
    Full,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct FunctionSignature {
    pub params: Vec<Type>,
    pub returns: Type,
}

impl FunctionSignature {
    pub fn new(params: Vec<Type>, returns: Type) -> Self {
        Self { params, returns }
    }
}

#[repr(C)]
#[derive(Debug, EnumDisplay, Default, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Linkage {
    /// Function Definition:  Visible outside module, single strong definition
    /// Function Declaration: External Symbol declaration
    #[default]
    #[display("external")]
    External,

    /// Function Definition:  Invisible outside module
    #[display("internal")]
    Internal,

    /// Multiple identical definitions allowed
    #[display("weak")]
    Weak,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, EnumDisplay, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum CallingConvention {
    /// C ABI
    #[default]
    #[display("c")]
    C,

    /// System V ABI, see https://refspecs.linuxbase.org/elf/x86_64-abi-0.99.pdf
    #[display("sysv")]
    SystemV,

    /// Microsoft ABI, see https://learn.microsoft.com/en-us/cpp/build/x64-calling-convention?view=msvc-170
    #[display("msabi")]
    MicrosoftAbi,
}

#[derive(Debug, EnumDisplay, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum IntCmp {
    #[display("eq")]
    Eq,
    #[display("ne")]
    Ne,
    #[display("lt")]
    Lt,
    #[display("le")]
    Le,
    #[display("gt")]
    Gt,
    #[display("ge")]
    Ge,
    #[display("ult")]
    ULt,
    #[display("ule")]
    ULe,
    #[display("ugt")]
    UGt,
    #[display("uge")]
    UGe,
}

#[derive(Debug, EnumDisplay, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum FloatCmp {
    #[display("ord")]
    Ord,
    #[display("oeq")]
    OEq,
    #[display("one")]
    ONe,
    #[display("olt")]
    OLt,
    #[display("ole")]
    OLe,
    #[display("ogt")]
    OGt,
    #[display("oge")]
    OGe,
    #[display("uno")]
    Uno,
    #[display("ueq")]
    UEq,
    #[display("une")]
    UNe,
    #[display("ult")]
    ULt,
    #[display("ule")]
    ULe,
    #[display("ugt")]
    UGt,
    #[display("uge")]
    UGe,
}
