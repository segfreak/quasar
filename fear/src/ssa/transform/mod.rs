use std::collections::HashMap;

use crate::{ssa::FunctionDef, tree, types::OptLevel};

use super::{FuncId, Module};

pub mod simplify;
pub mod cfg_simplify;
pub mod constfold;
pub mod copyprop;
pub mod cse;
pub mod dce;
pub mod dse;
pub mod gvn;
pub mod strength_reduction;
pub mod tre;
pub mod mem2reg;
pub mod tcf;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PassKind {
    ConstantFolding,
    CopyPropagation,
    DeadCodeElimination,
    StrengthReduction,
    CommonSubexpressionElimination,
    DeadStoreElimination,
    TailRecursionElimination,
    Simplify,
    GlobalValueNumbering,
    CFGSimplify,
    Mem2Reg,
    TrivialComparesFolding,
}

#[derive(Debug, Default)]
pub struct PassResult {
    pub changed: bool,
    pub passes: Vec<PassKind>,
}

pub struct PassManager;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PassManagerOpts {
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
        log::trace!("running optimiser with options: {:?}", opts);
        let mut result = PassResult::default();

        if opts.multilevel
            && let lvl = opts.multilevel_tree_level.unwrap_or(opts.level)
            && lvl >= OptLevel::Default
        {
            let old_def = m
                .get_function(f)
                .and_then(|func| func.get_definition())
                .cloned();

            if let Some(def) = old_def {
                let mut tdef = tree::FunctionDef::from(def);
                let before = tdef.get_cost();
                let res = tree::passes::PassManager::optimize_with_pipeline(
                    &tree::passes::Pipeline::with_level(opts.multilevel_tree_max_passes, lvl),
                    m,
                    &mut tdef,
                );
                let after = tdef.get_cost();
                log::debug!("tree optimiser performed {} passes ({lvl:?})", res.passes.len());
                log::debug!("is profitable: {}", before > after);
                let new_def = FunctionDef::from(tdef);
                if let Some(func) = m.get_function_mut(f)
                    && let Some(def_ref) = func.get_definition_mut()
                {
                    *def_ref = new_def;
                }
            }
        }

        let before_hash = m.get_function_mut(f).unwrap().get_definition().unwrap().get_hash();

        loop {
            let mut run = Vec::new();
            let before_hash = m.get_function_mut(f).unwrap().get_definition().unwrap().get_hash();

            macro_rules! run_pass {
                ($pass:expr, $kind:expr) => {
                    if $pass(m, f) {
                        log::trace!("performed pass: {:?}", $kind);
                        run.push($kind);
                        m.get_function_mut(f).unwrap().get_definition_mut().unwrap().reconstruct();
                    }
                };
            }

            if opts.level >= OptLevel::Default {
                run_pass!(constfold::constfold, PassKind::ConstantFolding);
                run_pass!(simplify::simplify, PassKind::Simplify);
                run_pass!(strength_reduction::strength_reduction, PassKind::StrengthReduction);
                run_pass!(copyprop::copyprop, PassKind::CopyPropagation);
                run_pass!(gvn::gvn, PassKind::GlobalValueNumbering);
                run_pass!(cse::cse, PassKind::CommonSubexpressionElimination);
                run_pass!(cfg_simplify::cfg_simplify, PassKind::CFGSimplify);
                run_pass!(tcf::tcf, PassKind::TrivialComparesFolding);
                run_pass!(dse::dse, PassKind::DeadStoreElimination);
                run_pass!(dce::dce, PassKind::DeadCodeElimination);
                run_pass!(tre::tre, PassKind::TailRecursionElimination);
                run_pass!(mem2reg::mem2reg, PassKind::Mem2Reg);
            }


            let changed = m.get_function_mut(f).unwrap().get_definition().unwrap().get_hash() != before_hash;
            if changed
            {
                log::debug!("pipeline: performed {:?} passes", run);
            }

            if !changed {
                break;
            }

            result.passes.extend(run);
        }

        result.changed = m.get_function_mut(f).unwrap().get_definition().unwrap().get_hash() != before_hash;

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

            if pass_result.changed
            {
                log::trace!("after passes:\n{}", m.dump_function(func_id));        
            }

            tmp.insert(func_id, pass_result);
        }

        tmp
    }
}

impl Module {
    pub fn optimize_with_options(&mut self, opts: PassManagerOpts) -> HashMap<FuncId, PassResult> {
        PassManager::run_module(&opts, self)
    }

    pub fn optimize(&mut self, level: OptLevel, multilevel: bool) -> HashMap<FuncId, PassResult> {
        let opts = PassManagerOpts {
            level,
            multilevel,
            multilevel_tree_max_passes: 128,
            multilevel_tree_level: None,
        };
        self.optimize_with_options(opts)
    }
}
