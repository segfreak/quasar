use clap::*;
use fear::style::*;
use fear::{ssa::Module, types::OptLevel};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "fearopt",
    author,
    version,
    about = "Fear binary-module optimizer",
    styles = styles(),
    propagate_version = true,
    arg_required_else_help = true,
)]
struct Cli {
    #[arg(help = "Input Fear Binary IR module (.bin)")]
    input: PathBuf,

    #[arg(
        short = 'o',
        help = "Output file path (defaults to stdout for text, file for objects)"
    )]
    output_path: Option<PathBuf>,

    #[arg(long = "opt", value_enum)]
    opt_level: Option<OptLevel>,

    #[arg(
        short = 'm',
        long = "multilevel",
        help = "Enable high-level Expression Tree optimizations"
    )]
    multilevel: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    let mut module = fear::binary::load_from_file::<Module>(&cli.input)
        .map_err(|e| format!("failed to load module file: {}", e))?;

    module.optimize(cli.opt_level.unwrap_or(OptLevel::Default), cli.multilevel);

    fear::binary::write_to_file::<Module>(&module, &cli.output_path.unwrap_or(cli.input))?;
    Ok(())
}
