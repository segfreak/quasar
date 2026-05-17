pub mod target;

use enum_display::EnumDisplay;

#[cfg(not(feature = "hashbrown"))]
pub use std::collections::HashMap;
#[cfg(not(feature = "hashbrown"))]
pub use std::collections::HashSet;

#[cfg(feature = "hashbrown")]
pub use hashbrown::HashMap;
#[cfg(feature = "hashbrown")]
pub use hashbrown::HashSet;

#[derive(Debug, EnumDisplay, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
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
            Self::Ptr => target::HOST_DESC.pointer_size,
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct FunctionSignature {
    pub params: Vec<Type>,
    pub returns: Type,
}

impl FunctionSignature {
    pub fn new(params: Vec<Type>, returns: Type) -> Self {
        Self { params, returns }
    }
}

#[derive(Debug, EnumDisplay, Default, Clone, PartialEq, Eq, Hash)]
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
