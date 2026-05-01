use crate::ir::{FuncId, Module};
use crate::prelude::*;

pub mod algebraic_simplify;
pub mod constant_folding;
pub mod copy_propogation;
pub mod cse;
pub mod dce;
pub mod dse;
pub mod gvn;
pub mod strength_reduction;
pub mod tre;
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
    TailRecursionElimination,
    AlgebraicSimplify,
    GlobalValueNumbering,
}

#[derive(Debug, Default)]
pub struct PassResult {
    pub changed: bool,
    pub passes: Vec<PassKind>,
}

pub struct PassManager;

impl PassManager {
    pub fn run_function(m: &mut Module, f: FuncId) -> PassResult {
        let mut result = PassResult::default();

        loop {
            let mut changed = false;
            let mut run = Vec::new();

            macro_rules! run_pass {
                ($pass:expr, $kind:expr) => {
                    if $pass(m, f) {
                        log::trace!("performed pass: {:?}", $kind);
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
                algebraic_simplify::algebraic_simplify,
                PassKind::AlgebraicSimplify
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
            run_pass!(gvn::gvn, PassKind::GlobalValueNumbering);
            run_pass!(dse::dse, PassKind::DeadStoreElimination);
            run_pass!(uce::uce, PassKind::UnreachableElimination);
            run_pass!(tre::tre, PassKind::TailRecursionElimination);

            result.passes.extend(run);

            if !changed {
                break;
            }

            result.changed = true;
        }

        if result.changed {
            let func = m.get_function_mut(f).unwrap().get_definition_mut().unwrap();
            func.reconstruct();
            func.full_rebuild();
        }

        result
    }

    pub fn run_module(m: &mut Module) -> HashMap<FuncId, PassResult> {
        let mut tmp = HashMap::new();

        let func_ids: Vec<FuncId> = m
            .iter_functions()
            .filter_map(|(id, f)| {
                if f.get_definition().is_some() {
                    Some(id)
                } else {
                    None
                }
            })
            .collect();

        for func_id in func_ids {
            let pass_result = PassManager::run_function(m, func_id);

            let func_name = &m.functions[&func_id].name;

            log::debug!(
                "optimiser has performed {} passes for the function {}: {:?}",
                pass_result.passes.len(),
                func_name,
                pass_result.passes
            );

            tmp.insert(func_id, pass_result);
        }

        tmp
    }
}

impl Module {
    pub fn optimize(&mut self) {
        PassManager::run_module(self);
    }
}
