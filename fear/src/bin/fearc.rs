use clap::*;
use fear::style::*;
use fear::{compiler::*, ssa::Module, types::OptLevel};
use std::str::FromStr;
use std::{fs::File, path::PathBuf};
use target_lexicon::Triple;

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
    #[arg(help = "Input Fear Binary IR module (.bin)")]
    input: PathBuf,

    #[arg(short = 'b', long = "backend", value_enum)]
    backend: Option<Backend>,

    #[arg(long = "type", value_enum)]
    output_type: Option<OutputType>,

    #[arg(
        short = 'o',
        help = "Output file path (defaults to stdout for text, file for objects)"
    )]
    output_path: Option<PathBuf>,

    #[arg(
        short = 't',
        long = "triple", 
        value_parser = |s: &str| target_lexicon::Triple::from_str(s).map_err(|e| e.to_string()),
        help = "Target platform triple (defaults to host)"
    )]
    triple: Option<Triple>,

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

    let output_type = cli.output_type.unwrap_or(OutputType::Object);
    let config = CompilerConfig {
        backend: cli
            .backend
            .unwrap_or(Backend::select_for(output_type).unwrap_or(Backend::Dummy)),
        output_type,
        triple: cli.triple.unwrap_or_else(Triple::host),
        opt_level: cli.opt_level.unwrap_or(OptLevel::Default),
    };

    let mut module = fear::binary::load_from_file::<Module>(&cli.input)
        .map_err(|e| format!("failed to load module file: {}", e))?;
    module.optimize(config.opt_level, cli.multilevel);

    let writer: Box<dyn std::io::Write> = match cli.output_path {
        Some(path) => Box::new(
            File::create(&path)
                .map_err(|_| format!("failed to open/create file: {}", path.to_string_lossy()))?,
        ),

        None => {
            if config.output_type.is_text() {
                Box::new(std::io::stdout())
            } else {
                let default_path =
                    PathBuf::from(&module.name).with_extension(config.output_type.extenstion());

                Box::new(File::create(&default_path).map_err(|_| {
                    format!(
                        "failed to open/create file: {}",
                        default_path.to_string_lossy()
                    )
                })?)
            }
        }
    };

    compile_module(&module, &config, writer)?;
    Ok(())
}
