use clap::*;
use fear::ir::Module;

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum OutputType {
    LlvmIr,
    Object,
}

impl OutputType {
    pub fn extenstion(&self) -> &'static str {
        match self {
            Self::LlvmIr => "ll",
            Self::Object => "o",
        }
    }
}

pub fn name(modname: &str, out_ty: OutputType) -> String {
    format!("{}.{}", modname, out_ty.extenstion())
}

#[derive(Parser)]
struct Cli {
    #[arg()]
    input: String,
    #[arg(short = 't', long = "type", value_enum)]
    ty: OutputType,
    #[arg(short = 'o')]
    out: Option<String>,
}

fn main() {
    let cli = Cli::parse();

    let module = fear::binary::load_from_file::<Module>(&cli.input).unwrap();
    let outname = cli.out.unwrap_or(name(&module.name, cli.ty));
    match cli.ty {
        OutputType::LlvmIr => {
            #[cfg(feature = "llvm")]
            {
                use fear::lowering::llvm::LlvmLowerer;
                use inkwell::context::Context;
                let llvm_ctx = Context::create();
                let mut lowerer = LlvmLowerer::new(&llvm_ctx, "test");
                lowerer.lower_module(&m);
                let llvm_module = lowerer.get_module();
                fs::write(outname, llvm_module.print_to_string().to_str().unwrap())
                    .expect("fs::write error");
            }
            #[cfg(not(feature = "llvm"))]
            {
                todo!("llvm is not supported in this build")
            }
        }
        OutputType::Object => {
            #[cfg(feature = "cranelift")]
            {
                use cranelift::codegen::settings::Configurable;
                use cranelift_module::default_libcall_names;
                use fear::lowering::cranelift::CraneliftLowerer;
                use std::fs;

                let mut flag_builder = cranelift::codegen::settings::builder();
                flag_builder.set("use_colocated_libcalls", "false").unwrap();
                flag_builder.set("is_pic", "true").unwrap();
                let flags = cranelift::codegen::settings::Flags::new(flag_builder);
                let isa = cranelift::codegen::isa::lookup(target_lexicon::Triple::host())
                    .unwrap()
                    .finish(flags)
                    .unwrap();

                let mut lowerer = CraneliftLowerer::new(isa, default_libcall_names(), "test");
                lowerer.lower_module(&module);
                let object_bytes = lowerer.finish();
                fs::write(outname, object_bytes).expect("fs::write error");
            }
            #[cfg(not(feature = "cranelift"))]
            {
                todo!("cranelift is not supported in this build")
            }
        }
    }
}
