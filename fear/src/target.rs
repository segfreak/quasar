pub const HOST_DESC: TargetDescription = host_description();

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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
