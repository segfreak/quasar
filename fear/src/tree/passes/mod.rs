pub mod canonicalize;
pub mod cse;
pub mod dce;
pub mod expressify;
pub mod fold;
pub mod normalize;
pub mod simplify;
pub mod strength_reduction;

use std::collections::HashSet;

use crate::tree::*;
use crate::types::OptLevel;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PassKind {
    CommonSubexpressionElimination,
    DeadCodeElimination,
    Expressify,
    ConstantFolding,
    Simplify,
    StrengthReduction,
    Normalize,
    Canonicalize,
}

#[derive(Debug, Default)]
pub struct PassResult {
    pub changed: bool,
    pub passes: Vec<PassKind>,
}

pub struct PassManager;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pipeline {
    max_passes: i32,
    passes: HashSet<PassKind>,
}

impl Pipeline {
    pub fn new(max_passes: i32) -> Self {
        Self {
            max_passes,
            passes: HashSet::new(),
        }
    }

    pub fn with_passes(max_passes: i32, passes: &[PassKind]) -> Self {
        use PassKind::*;

        let mut pipeline = Self::new(max_passes);

        pipeline.get_passes_mut().insert(Expressify);
        pipeline.passes.extend(passes);
        pipeline
    }

    pub fn with_level(max_passes: i32, level: OptLevel) -> Self {
        use PassKind::*;

        let mut pipeline = Self::new(max_passes);

        pipeline.get_passes_mut().insert(Expressify);

        if level <= OptLevel::Default {
            pipeline.passes.extend(vec![
                CommonSubexpressionElimination,
                DeadCodeElimination,
                Expressify,
                ConstantFolding,
                Simplify,
                StrengthReduction,
                Normalize,
                Canonicalize,
            ]);
        }

        pipeline
    }

    pub fn get_max_passes(&self) -> i32 {
        self.max_passes
    }

    pub fn get_passes(&self) -> &HashSet<PassKind> {
        &self.passes
    }

    pub fn get_passes_mut(&mut self) -> &mut HashSet<PassKind> {
        &mut self.passes
    }

    pub fn has_pass(&self, pass: PassKind) -> bool {
        self.passes.contains(&pass)
    }
}

impl PassManager {
    pub fn optimize_with_pipeline(
        pipeline: &Pipeline,
        // fear::ssa module (because fear::tree lowers into fear::ssa)
        m: &crate::ssa::Module,
        // treessa function def, can call only fear function that declared or defined on 'm'
        f: &mut FunctionDef,
    ) -> PassResult {
        let mut result = PassResult::default();

        let before_cost = f.get_cost();

        let mut counter: i32 = 0;
        loop {
            let before_cost = f.get_cost();
            let before_hash = f.dirty_hash();

            let mut run = Vec::new();

            macro_rules! run_pass {
                ($pass:expr, $kind:expr) => {
                    if $pass(f) {
                        log::trace!("performed pass: {:?}", $kind);
                        run.push($kind);
                    }
                };
                ($module:expr, $pass:expr, $kind:expr) => {
                    if $pass($module, f) {
                        log::trace!("performed pass: {:?}", $kind);
                        run.push($kind);
                    }
                };
            }

            macro_rules! try_run_pass {
                ($pass:expr, $kind:expr) => {
                    if pipeline.has_pass($kind) {
                        run_pass!($pass, $kind)
                    }
                };
                ($module:expr, $pass:expr, $kind:expr) => {
                    if pipeline.has_pass($kind) {
                        run_pass!($module, $pass, $kind)
                    }
                };
            }

            try_run_pass!(expressify::expressify, PassKind::Expressify);

            try_run_pass!(normalize::normalize, PassKind::Normalize);
            try_run_pass!(simplify::simplify, PassKind::Simplify);
            try_run_pass!(fold::fold, PassKind::ConstantFolding);
            try_run_pass!(canonicalize::canonicalize, PassKind::Canonicalize);
            try_run_pass!(
                strength_reduction::strength_reduction,
                PassKind::StrengthReduction
            );
            try_run_pass!(m, cse::cse, PassKind::CommonSubexpressionElimination);
            try_run_pass!(dce::dce, PassKind::DeadCodeElimination);

            result.passes.extend(run);
            counter += 1;

            let after_cost = f.get_cost();
            if after_cost > before_cost {
                log::warn!(
                    "optimization regression detected: cost increased from {} to {} (+{})",
                    before_cost,
                    after_cost,
                    after_cost - before_cost
                );
            }
            let changed = f.dirty_hash() != before_hash;
            if !changed || counter >= pipeline.max_passes {
                break;
            }

            result.changed = true;
        }

        let after_cost = f.get_cost();

        log::debug!(
            "summary: cost {} -> {} (diff: -{}, improvement: {:.2}%)",
            before_cost,
            after_cost,
            before_cost.saturating_sub(after_cost),
            if before_cost > 0 {
                (before_cost as f64 - after_cost as f64) / before_cost as f64 * 100.0
            } else {
                0.0
            }
        );

        log::debug!("after passes:\n{}", f.dump());

        result
    }
}
