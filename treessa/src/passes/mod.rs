pub mod cse;
pub mod dead_code_elim;
pub mod expressify;
pub mod fold;
pub mod simplify;

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
    pub fn run_function(f: &mut FunctionDef) -> PassResult {
        let mut result = PassResult::default();

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
            run_pass!(fold::fold, PassKind::ConstantFolding);
            run_pass!(simplify::simplify, PassKind::Simplify);
            // run_pass!(
            //     strength_reduction::strength_reduction,
            //     PassKind::StrengthReduction
            // );
            run_pass!(cse::cse, PassKind::CommonSubexpressionElimination);
            run_pass!(
                dead_code_elim::dead_code_elim,
                PassKind::DeadCodeElimination
            );
            run_pass!(expressify::expressify, PassKind::Expressify);

            result.passes.extend(run);

            if !changed {
                break;
            }

            result.changed = true;
        }

        result
    }
}
