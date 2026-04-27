pub const HOST_DESC: TargetDescription = host_description();

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TargetDescription {
    pub pointer_size: usize,
}

impl TargetDescription {
    pub fn host() -> Self {
        host_description()
    }
}

impl Default for TargetDescription {
    fn default() -> Self {
        Self::host()
    }
}

#[cfg(target_arch = "x86_64")]
const fn host_description() -> TargetDescription {
    TargetDescription { pointer_size: 8 }
}

#[derive(Debug, Default)]
pub enum CallingConvention {
    /// C ABI
    #[default]
    C,

    /// System V ABI, see https://refspecs.linuxbase.org/elf/x86_64-abi-0.99.pdf
    SystemV,

    /// Microsoft ABI, see https://learn.microsoft.com/en-us/cpp/build/x64-calling-convention?view=msvc-170
    MicrosoftAbi,
}
