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
    Int1,
    #[display("i8")]
    Int8,
    #[display("i16")]
    Int16,
    #[display("i32")]
    Int32,
    #[display("i64")]
    Int64,
    #[display("f32")]
    Float32,
    #[display("f64")]
    Float64,
    #[display("ptr")]
    Pointer,
}

impl Type {
    pub fn is_integer(&self) -> bool {
        matches!(self, Type::Int8 | Type::Int16 | Type::Int32 | Type::Int64)
    }

    pub fn is_float(&self) -> bool {
        matches!(self, Type::Float32 | Type::Float64)
    }

    pub fn is_void(&self) -> bool {
        matches!(self, Type::Void)
    }

    pub fn is_pointer(&self) -> bool {
        matches!(self, Type::Pointer)
    }

    pub fn get_size(&self) -> usize {
        match self {
            Self::Void => 0,
            Self::Int1 => 1,
            Self::Int8 => 1,
            Self::Int16 => 2,
            Self::Int32 => 4,
            Self::Int64 => 8,
            Self::Float32 => 4,
            Self::Float64 => 8,
            Self::Pointer => crate::target::HOST_DESC.pointer_size,
        }
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
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

#[derive(Debug, EnumDisplay, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum CastKind {
    #[display("zext")]
    Zext,
    #[display("sext")]
    Sext,
    #[display("trunc")]
    Trunc,
    #[display("bitcast")]
    Bitcast,

    /// Signed integer to float-point
    #[display("s2f")]
    SIToFP,
    /// Unsigned integer to float-point
    #[display("u2f")]
    UIToFP,

    /// Float-point to signed integer
    #[display("f2s")]
    FPToSI,
    /// Float-point to unsigned integer
    #[display("f2u")]
    FPToUI,

    /// promotes float precision, for example: Float32 -> Float64
    #[display("fprom")]
    FPromote,
    /// truncates float precision, for example: Float64 -> Float32
    #[display("ftrunc")]
    FTrunc,
}
