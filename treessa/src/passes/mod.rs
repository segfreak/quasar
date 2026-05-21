pub mod cse;
pub mod dce;
pub mod expressify;
pub mod fold;
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
}

#[derive(Debug, Default)]
pub struct PassResult {
    pub changed: bool,
    pub passes: Vec<PassKind>,
}

pub struct PassManager;

impl PassManager {
    pub fn optimize(f: &mut FunctionDef, level: OptLevel) -> PassResult {
        let mut result = PassResult::default();

        let before_cost = f.get_cost();

        loop {
            let mut changed = false;
            let mut run = Vec::new();

            macro_rules! run_pass {
                ($pass:expr, $kind:expr) => {
                    if $pass(f) {
                        log::trace!("performed pass: {:?}", $kind);
                        changed = true;
                        run.push($kind);
                    }
                };
            }

            run_pass!(expressify::expressify, PassKind::Expressify);

            if level >= OptLevel::Default {
                run_pass!(fold::fold, PassKind::ConstantFolding);
                run_pass!(simplify::simplify, PassKind::Simplify);
                run_pass!(
                    strength_reduction::strength_reduction,
                    PassKind::StrengthReduction
                );
                run_pass!(cse::cse, PassKind::CommonSubexpressionElimination);
                run_pass!(dce::dce, PassKind::DeadCodeElimination);
                run_pass!(expressify::expressify, PassKind::Expressify);
            }

            result.passes.extend(run);

            if !changed {
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
