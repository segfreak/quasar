use std::str::FromStr;
use std::{fs::File, path::PathBuf};

use clap::builder::styling::{AnsiColor, Effects};
use clap::builder::Styles;
use clap::*;
use fear::{compiler::*, ssa::Module, types::OptLevel};
use target_lexicon::Triple;

fn styles() -> Styles {
    Styles::styled()
        .header(AnsiColor::BrightBlue.on_default() | Effects::BOLD)
        .usage(AnsiColor::BrightGreen.on_default() | Effects::BOLD)
        .literal(AnsiColor::BrightCyan.on_default())
        .placeholder(AnsiColor::BrightMagenta.on_default())
}

#[derive(Parser, Debug)]
#[command(
    name = "fearc",
    author,
    version,
    about = "Fear binary-module compiler driver",
    long_about = "Compile Fear Binary IR modules into machine code.",
    styles = styles(),
    propagate_version = true,
    arg_required_else_help = true,
)]
struct Cli {
    #[arg()]
    input: String,
    #[arg(short = 'b', long = "backend", value_enum)]
    backend: Option<Backend>,
    #[arg(long = "type", value_enum)]
    output_type: Option<OutputType>,
    #[arg(short = 'o')]
    output_path: Option<PathBuf>,
    #[arg(
        short = 't',
        long = "triple", 
        value_parser = |s: &str| target_lexicon::Triple::from_str(s).map_err(|e| e.to_string())
    )]
    triple: Option<Triple>,
    #[arg(long = "opt")]
    opt_level: Option<OptLevel>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    let config = CompilerConfig {
        backend: cli.backend.unwrap_or(Backend::Cranelift),
        output_type: cli.output_type.unwrap_or(OutputType::Object),
        triple: cli.triple.unwrap_or_else(Triple::host),
        opt_level: cli.opt_level.unwrap_or(OptLevel::Default),
    };

    let module = fear::binary::load_from_file::<Module>(&cli.input)
        .map_err(|e| format!("failed to load module file: {}", e))?;

    let output_path = cli.output_path.unwrap_or_else(|| {
        PathBuf::from(&module.name).with_extension(config.output_type.extenstion())
    });

    let file = File::create(&output_path).map_err(|_e| {
        format!(
            "failed to open/create file: {}",
            output_path.to_string_lossy()
        )
    })?;

    compile_module(&module, &config, file)?;
    Ok(())
}
