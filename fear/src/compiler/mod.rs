use crate::ssa::Module;
use crate::types::OptLevel;
use clap::ValueEnum;
use enum_display::EnumDisplay;
use std::io::Write;
use target_lexicon::Triple;

#[cfg(feature = "llvm")]
use inkwell::OptimizationLevel;

#[derive(Debug, EnumDisplay, Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum OutputType {
    #[cfg(feature = "llvm")]
    #[display("llvm-ir")]
    LlvmIr,
    #[display("assembly")]
    Assembly,
    #[default]
    #[display("object")]
    Object,
}

/// Backend kind
#[derive(Debug, EnumDisplay, Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum Backend {
    #[default]
    #[cfg(feature = "cranelift")]
    #[display("cranelift")]
    Cranelift,
    #[display("fear")]
    Fear,
    #[cfg(feature = "llvm")]
    #[display("llvm")]
    Llvm,
}

impl Backend {
    pub fn select_for(output_type: OutputType) -> Option<Self> {
        match output_type {
            #[cfg(feature = "llvm")]
            OutputType::LlvmIr => Some(Self::Llvm),

            OutputType::Assembly => {
                #[cfg(feature = "llvm")]
                {
                    Some(Self::Llvm)
                }
                #[cfg(not(feature = "llvm"))]
                {
                    None
                }
            }

            OutputType::Object => {
                #[cfg(feature = "cranelift")]
                {
                    Some(Self::Cranelift)
                }
                #[cfg(all(not(feature = "cranelift"), feature = "llvm"))]
                {
                    Some(Self::Llvm)
                }
                #[cfg(all(not(feature = "cranelift"), not(feature = "llvm")))]
                {
                    None
                }
            }
        }
    }

    pub fn supported_output_types(&self) -> &[OutputType] {
        match self {
            #[cfg(feature = "llvm")]
            Self::Llvm => &[OutputType::LlvmIr, OutputType::Assembly, OutputType::Object],
            #[cfg(feature = "cranelift")]
            Self::Cranelift => &[OutputType::Object],
            Self::Fear => &[],
        }
    }
}

impl OutputType {
    pub fn extenstion(&self) -> &'static str {
        match self {
            #[cfg(feature = "llvm")]
            Self::LlvmIr => "ll",
            Self::Assembly => "s",
            Self::Object => "o",
        }
    }
}

#[cfg(feature = "llvm")]
impl From<OptLevel> for OptimizationLevel {
    fn from(value: OptLevel) -> Self {
        match value {
            OptLevel::None => Self::None,
            OptLevel::Default => Self::Default,
            OptLevel::Full => Self::Aggressive,
        }
    }
}

#[cfg(feature = "cranelift")]
impl From<OptLevel> for cranelift::codegen::settings::OptLevel {
    fn from(value: OptLevel) -> Self {
        match value {
            OptLevel::None => Self::None,
            OptLevel::Default => Self::Speed,
            OptLevel::Full => Self::SpeedAndSize,
        }
    }
}

pub fn has_backend(backend: Backend) -> bool {
    match backend {
        #[cfg(feature = "llvm")]
        Backend::Llvm => crate::ssa::lowering::has_llvm(),
        #[cfg(feature = "cranelift")]
        Backend::Cranelift => crate::ssa::lowering::has_cranelift(),
        Backend::Fear => {
            /* not yet implemented */
            false
        }
    }
}

#[derive(Debug, Clone)]
pub struct CompilerConfig {
    pub backend: Backend,
    pub output_type: OutputType,
    pub triple: Triple,
    pub opt_level: OptLevel,
}

impl Default for CompilerConfig {
    fn default() -> Self {
        Self {
            backend: Backend::Cranelift,
            output_type: OutputType::Object,
            triple: Triple::host(),
            opt_level: OptLevel::Default,
        }
    }
}

impl CompilerConfig {
    pub fn setup(output_type: OutputType, triple: Triple, opt_level: OptLevel) -> Self {
        let error_msg = format!("cannot select backend for {}", output_type);
        let backend = Backend::select_for(output_type).expect(&error_msg);
        Self {
            backend,
            output_type,
            triple,
            opt_level,
        }
    }

    pub(crate) fn is_valid(&self) -> bool {
        let output_is_supported = self
            .backend
            .supported_output_types()
            .contains(&self.output_type);
        if !output_is_supported {
            log::error!(
                "output type {} is unsupported by {} backend",
                self.output_type,
                self.backend
            );
        }

        output_is_supported
    }
}

#[allow(unused)]
pub fn compile_module<W: Write>(
    module: &Module,
    config: &CompilerConfig,
    mut writer: W,
) -> Result<(), Box<dyn std::error::Error>> {
    if !config.is_valid() {
        log::error!("compiling using invalid compiler-config");
    }

    match (config.backend, config.output_type) {
        #[cfg(feature = "llvm")]
        (Backend::Llvm, ty) => {
            #[cfg(feature = "llvm")]
            {
                use crate::ssa::lowering::llvm::LlvmLowerer;
                use inkwell::context::Context;

                let llvm_ctx = Context::create();
                let mut lowerer = LlvmLowerer::new(&module.name, config.triple.clone(), &llvm_ctx);
                lowerer.lower_module(module);
                let llvm_module = lowerer.get_module();
                let target_machine = lowerer.get_target_machine();

                llvm_module
                    .run_passes(
                        match OptimizationLevel::from(config.opt_level) {
                            OptimizationLevel::None => "default<O0>",
                            OptimizationLevel::Less => "default<O1>",
                            OptimizationLevel::Default => "default<O2>",
                            OptimizationLevel::Aggressive => "default<O3>",
                        },
                        target_machine,
                        inkwell::passes::PassBuilderOptions::create(),
                    )
                    .unwrap();

                match ty {
                    OutputType::LlvmIr => {
                        let ir_string = llvm_module.print_to_string().to_string();
                        writer.write_all(ir_string.as_bytes())?;
                    }
                    OutputType::Assembly => {
                        let buffer = target_machine
                            .write_to_memory_buffer(
                                llvm_module,
                                inkwell::targets::FileType::Assembly,
                            )
                            .map_err(|e| format!("llvm target machine error: {:?}", e))?;
                        writer.write_all(buffer.as_slice())?;
                    }
                    OutputType::Object => {
                        let buffer = target_machine
                            .write_to_memory_buffer(llvm_module, inkwell::targets::FileType::Object)
                            .map_err(|e| format!("llvm target machine error: {:?}", e))?;
                        writer.write_all(buffer.as_slice())?;
                    }
                }
                writer.flush()?;
                Ok(())
            }
            #[cfg(not(feature = "llvm"))]
            {
                Err("llvm backend is not supported in this build. recompile with --features llvm,inkwell/llvmXX-X".into())
            }
        }

        #[cfg(feature = "cranelift")]
        (Backend::Cranelift, OutputType::Object) => {
            #[cfg(feature = "cranelift")]
            {
                use crate::ssa::lowering::cranelift::CraneliftLowerer;
                use cranelift::codegen::settings::{self, Configurable};
                use cranelift_module::default_libcall_names;

                let mut flag_builder = cranelift::codegen::settings::builder();
                flag_builder
                    .set("use_colocated_libcalls", "false")
                    .map_err(|e| format!("cranelift config error: {:?}", e))?;
                flag_builder
                    .set("is_pic", "true")
                    .map_err(|e| format!("cranelift config error: {:?}", e))?;
                flag_builder
                    .set(
                        "opt_level",
                        &settings::OptLevel::from(config.opt_level).to_string(),
                    )
                    .map_err(|e| format!("cranelift config error: {:?}", e))?;

                let flags = cranelift::codegen::settings::Flags::new(flag_builder);
                let isa = cranelift::codegen::isa::lookup(config.triple.clone())
                    .map_err(|e| format!("unsupported target triple: {}", e))?
                    .finish(flags)
                    .map_err(|e| format!("failed to initialize cranelift isa: {}", e))?;

                let mut lowerer = CraneliftLowerer::new(&module.name, isa, default_libcall_names());
                lowerer.lower_module(module);
                let object_bytes = lowerer.finish();

                writer.write_all(&object_bytes)?;
                writer.flush()?;
                Ok(())
            }
            #[cfg(not(feature = "cranelift"))]
            {
                Err("cranelift backend is not supported in this build. recompile with --features cranelift".into())
            }
        }

        (b, t) => Err(format!("output type {:?} is not supported by backend {:?}", t, b).into()),
    }
}
