use clap::*;
use fear::ssa::Module;
use fear::style::*;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "feardump",
    author,
    version,
    about = "Fear binary-module dumper (in text format)",
    styles = styles(),
    propagate_version = true,
    arg_required_else_help = true,
)]
struct Cli {
    #[arg(help = "Input Fear Binary IR module (.bin)")]
    input: PathBuf,

    #[arg(
        short = 'o',
        long = "output",
        help = "Write output to file instead of stdout"
    )]
    output: Option<PathBuf>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    let module = fear::binary::load_from_file::<Module>(&cli.input)
        .map_err(|e| format!("failed to load module file: {}", e))?;

    let mut writer: Box<dyn Write> = match cli.output {
        Some(path) => Box::new(
            File::create(&path)
                .map_err(|_| format!("failed to create output file: {}", path.to_string_lossy()))?,
        ),
        None => Box::new(std::io::stdout()),
    };

    writeln!(writer, "{}", module.dump())?;
    Ok(())
}
