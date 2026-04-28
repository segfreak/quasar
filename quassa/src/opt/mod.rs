use crate::ir::{FunctionDef, Module};

pub mod constant_folding;
pub mod copy_propogation;
pub mod cse;
pub mod dce;
pub mod dse;
pub mod strength_reduction;
pub mod uce;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PassKind {
    ConstantFolding,
    CopyPropagation,
    DeadCodeElimination,
    StrengthReduction,
    CommonSubexpressionElimination,
    DeadStoreElimination,
    UnreachableElimination,
}

#[derive(Debug, Default)]
pub struct PassResult {
    pub changed: bool,
    pub passes: Vec<PassKind>,
}

pub struct PassManager;

impl PassManager {
    pub fn run_function(f: &mut FunctionDef) -> PassResult {
        let mut result = PassResult::default();

        loop {
            let mut changed = false;
            let mut run = Vec::new();

            macro_rules! run_pass {
                ($pass:expr, $kind:expr) => {
                    if $pass(f) {
                        changed = true;
                        run.push($kind);
                    }
                };
            }

            run_pass!(
                constant_folding::constant_folding,
                PassKind::ConstantFolding
            );
            run_pass!(
                copy_propogation::copy_propogation,
                PassKind::CopyPropagation
            );
            run_pass!(dce::dce, PassKind::DeadCodeElimination);
            run_pass!(
                strength_reduction::strength_reduction,
                PassKind::StrengthReduction
            );
            run_pass!(cse::cse, PassKind::CommonSubexpressionElimination);
            run_pass!(dse::dse, PassKind::DeadStoreElimination);
            run_pass!(uce::uce, PassKind::UnreachableElimination);

            result.passes.extend(run);

            if !changed {
                break;
            }

            result.changed = true;
        }

        if result.changed {
            f.reconstruct();
        }

        result
    }
}

pub fn perform(module: &mut Module) {
    for (_, func) in module.iter_functions_mut() {
        if let Some(f) = func.get_definition_mut() {
            let PassResult { passes, .. } = PassManager::run_function(f);
            log::debug!(
                "optimiser has performed {} passes for the function {}: {:?}",
                passes.len(),
                func.name,
                passes
            );
        }
    }
}

impl Module {
    pub fn optimize(&mut self) {
        perform(self);
    }
}
