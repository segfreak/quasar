use clap::ValueEnum;
#[derive(Debug, Clone, Copy, ValueEnum)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum OptLevel {
    None,
    Default,
    Full,
}
