use std::collections::HashMap;

use crate::{ssa::FunctionDef, tree, types::OptLevel};

use super::{FuncId, Module};

pub mod algebraic_simplify;
pub mod cfg_simplify;
pub mod constfold;
pub mod copyprop;
pub mod cse;
pub mod dce;
pub mod dse;
pub mod gvn;
pub mod strength_reduction;
pub mod tre;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PassKind {
    ConstantFolding,
    CopyPropagation,
    DeadCodeElimination,
    StrengthReduction,
    CommonSubexpressionElimination,
    DeadStoreElimination,
    TailRecursionElimination,
    AlgebraicSimplify,
    GlobalValueNumbering,
    CFGSimplify,
}

#[derive(Debug, Default)]
pub struct PassResult {
    pub changed: bool,
    pub passes: Vec<PassKind>,
}

pub struct PassManager;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PassManagerOpts
{
    // optimization level
    pub level: OptLevel,
    // enable multilevel optimization pipeline
    // pipeline: fear::ssa -> fear::tree (opt) -> fear::ssa (opt)
    pub multilevel: bool,
    // max passes for fear::tree optimizer pipeline
    pub multilevel_tree_max_passes: i32,
    // optimization level for fear::tree optimizer pipeline
    pub multilevel_tree_level: Option<OptLevel>,

}

impl PassManager {
    pub fn run_function(opts: &PassManagerOpts, m: &mut Module, f: FuncId) -> PassResult {
        let mut result = PassResult::default();

        if opts.multilevel && let lvl = opts.multilevel_tree_level.unwrap_or(opts.level) && lvl >= OptLevel::Default {
            let old_def = m
                .get_function(f)
                .and_then(|func| func.get_definition())
                .cloned();

            if let Some(def) = old_def {
                let mut tdef = tree::FunctionDef::from(def);
                let res = tree::passes::PassManager::optimize(m, &mut tdef, lvl, opts.multilevel_tree_max_passes);
                log::debug!("tree optimizer performed {} passes", res.passes.len());
                let new_def = FunctionDef::from(tdef);
                if let Some(func) = m.get_function_mut(f) && let Some(def_ref) = func.get_definition_mut()  {
                        *def_ref = new_def;
                }
            }
        }

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
                    if changed {
                        let func = m.get_function_mut(f).unwrap().get_definition_mut().unwrap();
                        func.reconstruct();
                    }
                };
            }

            if opts.level <= OptLevel::Default {
                run_pass!(constfold::constfold, PassKind::ConstantFolding);
                run_pass!(
                    algebraic_simplify::algebraic_simplify,
                    PassKind::AlgebraicSimplify
                );
                run_pass!(
                    strength_reduction::strength_reduction,
                    PassKind::StrengthReduction
                );
                run_pass!(copyprop::copyprop, PassKind::CopyPropagation);
                run_pass!(gvn::gvn, PassKind::GlobalValueNumbering);
                run_pass!(cse::cse, PassKind::CommonSubexpressionElimination);
                run_pass!(dse::dse, PassKind::DeadStoreElimination);
                run_pass!(tre::tre, PassKind::TailRecursionElimination);
                run_pass!(dce::dce, PassKind::DeadCodeElimination);
                run_pass!(cfg_simplify::cfg_simplify, PassKind::CFGSimplify);
            }

            result.passes.extend(run);

            if !changed {
                break;
            }

            result.changed = true;
        }

        result
    }

    pub fn run_module(opts: &PassManagerOpts, m: &mut Module) -> HashMap<FuncId, PassResult> {
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
            let pass_result = PassManager::run_function(opts, m, func_id);

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
    pub fn optimize(&mut self, level: OptLevel, multilevel: bool) -> HashMap<FuncId, PassResult> {
        let opts = PassManagerOpts {
            level,
            multilevel,
            multilevel_tree_max_passes: 128,
            multilevel_tree_level: None,
        };
        PassManager::run_module(&opts, self)
    }
}
