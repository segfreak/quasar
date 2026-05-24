pub mod cse;
pub mod dce;
pub mod expressify;
pub mod fold;
pub mod normalize;
pub mod simplify;
pub mod strength_reduction;

use fear::types::OptLevel;

use crate::FunctionDef;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PassKind {
    CommonSubexpressionElimination,
    DeadCodeElimination,
    Expressify,
    ConstantFolding,
    Simplify,
    StrengthReduction,
    Normalize,
}

#[derive(Debug, Default)]
pub struct PassResult {
    pub changed: bool,
    pub passes: Vec<PassKind>,
}

pub struct PassManager;

impl PassManager {
    pub fn optimize(f: &mut FunctionDef, level: OptLevel, max_passes: i32) -> PassResult {
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
            }

            run_pass!(expressify::expressify, PassKind::Expressify);

            if level >= OptLevel::Default {
                run_pass!(normalize::normalize, PassKind::Normalize);
                run_pass!(simplify::simplify, PassKind::Simplify);
                run_pass!(fold::fold, PassKind::ConstantFolding);
                run_pass!(
                    strength_reduction::strength_reduction,
                    PassKind::StrengthReduction
                );
                run_pass!(cse::cse, PassKind::CommonSubexpressionElimination);
                run_pass!(dce::dce, PassKind::DeadCodeElimination);
            }

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
            if !changed || counter >= max_passes {
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

        result
    }
}
