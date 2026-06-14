use clap::*;
use fear::ssa::Module;
use fear::style::*;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "fearopt",
    author,
    version,
    about = "Fear binary-module linker",
    styles = styles(),
    propagate_version = true,
    arg_required_else_help = true,
)]
struct Cli {
    #[arg(help = "Input Fear Binary IR modules (.bin)")]
    input: Vec<PathBuf>,

    #[arg(
        short = 'o',
        help = "Output file path (defaults to stdout for text, file for objects)"
    )]
    output_path: Option<PathBuf>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    let mut modules = Vec::new();
    for path in &cli.input {
        let m = fear::binary::load_from_file::<Module>(path).map_err(|e| {
            format!(
                "failed to load module file '{}': {}",
                path.to_string_lossy(),
                e
            )
        })?;
        modules.push(m);
    }

    let out_path = cli
        .output_path
        .clone()
        .unwrap_or_else(|| cli.input[0].with_extension("bin"));

    let output_module_name = out_path
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| "linked_module".to_string());

    let result = match fear::linker::link(output_module_name, modules) {
        Ok(m) => m,
        Err(errors) => {
            let mut report = String::from("linkage errors:\n");
            for err in errors {
                report.push_str(&format!("  - {}\n", err));
            }
            return Err(report.into());
        }
    };

    fear::binary::write_to_file::<Module>(&result.module, &out_path)
        .map_err(|e| format!("failed to write linked module: {}", e))?;

    Ok(())
}
