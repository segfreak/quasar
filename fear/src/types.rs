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

impl TryFrom<&str> for Type {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "void" => Ok(Type::Void),
            "bool" => Ok(Type::Int1),
            "i8" => Ok(Type::Int8),
            "i16" => Ok(Type::Int16),
            "i32" => Ok(Type::Int32),
            "i64" => Ok(Type::Int64),
            "f32" => Ok(Type::Float32),
            "f64" => Ok(Type::Float64),
            "ptr" => Ok(Type::Pointer),
            _ => Err(format!("Unknown type: '{value}'")),
        }
    }
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

    /// Get size in bytes
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

    /// Get size in bits
    pub fn get_bitwidth(&self) -> usize {
        self.get_size() * 8
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

impl TryFrom<&str> for Linkage {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "external" => Ok(Self::External),
            "internal" => Ok(Self::Internal),
            "weak" => Ok(Self::Weak),
            _ => Err(format!("Unknown linkage type: '{value}'")),
        }
    }
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

impl TryFrom<&str> for CallingConvention {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "c" => Ok(Self::C),
            "sysv" => Ok(Self::SystemV),
            "msabi" => Ok(Self::MicrosoftAbi),
            _ => Err(format!("Unknown calling convention: '{value}'")),
        }
    }
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

impl TryFrom<&str> for IntCmp {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "eq" => Ok(Self::Eq),
            "ne" => Ok(Self::Ne),
            "lt" => Ok(Self::Lt),
            "le" => Ok(Self::Le),
            "gt" => Ok(Self::Gt),
            "ge" => Ok(Self::Ge),
            "ult" => Ok(Self::ULt),
            "ule" => Ok(Self::ULe),
            "ugt" => Ok(Self::UGt),
            "uge" => Ok(Self::UGe),
            _ => Err(format!("Unknown integer comparison operator: '{value}'")),
        }
    }
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

impl TryFrom<&str> for FloatCmp {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "ord" => Ok(Self::Ord),
            "oeq" => Ok(Self::OEq),
            "one" => Ok(Self::ONe),
            "olt" => Ok(Self::OLt),
            "ole" => Ok(Self::OLe),
            "ogt" => Ok(Self::OGt),
            "oge" => Ok(Self::OGe),
            "uno" => Ok(Self::Uno),
            "ueq" => Ok(Self::UEq),
            "une" => Ok(Self::UNe),
            "ult" => Ok(Self::ULt),
            "ule" => Ok(Self::ULe),
            "ugt" => Ok(Self::UGt),
            "uge" => Ok(Self::UGe),
            _ => Err(format!("Unknown float comparison operator: '{value}'")),
        }
    }
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
    #[display("fpromote")]
    FPromote,
    /// truncates float precision, for example: Float64 -> Float32
    #[display("ftrunc")]
    FTrunc,
}

impl TryFrom<&str> for CastKind {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "zext" => Ok(Self::Zext),
            "sext" => Ok(Self::Sext),
            "trunc" => Ok(Self::Trunc),
            "bitcast" => Ok(Self::Bitcast),
            "s2f" => Ok(Self::SIToFP),
            "u2f" => Ok(Self::UIToFP),
            "f2s" => Ok(Self::FPToSI),
            "f2u" => Ok(Self::FPToUI),
            "fpromote" => Ok(Self::FPromote),
            "ftrunc" => Ok(Self::FTrunc),
            _ => Err(format!("Unknown cast kind: '{value}'")),
        }
    }
}
