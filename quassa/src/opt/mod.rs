use crate::ir::{FunctionDef, Module};

pub mod cfg_simplify;
pub mod constant_folding;
pub mod copy_propogation;
pub mod cse;
pub mod dce;
pub mod dse;
pub mod strength_reduction;

/// performs optimizations for function
pub fn perform_for(f: &mut FunctionDef) {
    loop {
        let before = f.insts.len();

        constant_folding::constant_folding(f);
        copy_propogation::copy_propogation(f);
        dce::dce(f);
        strength_reduction::strength_reduction(f);
        cse::cse(f);
        dse::dse(f);
        cfg_simplify::cfg_simplify(f);

        f.reconstruct();

        if f.insts.len() == before {
            break;
        }
    }
}

pub fn perform(module: &mut Module) {
    for (_, func) in module.iter_functions_mut() {
        if let Some(f) = func.get_definition_mut() {
            perform_for(f);
        }
    }
}

impl FunctionDef {
    pub fn optimize(&mut self) {
        perform_for(self);
    }
}

impl Module {
    pub fn optimize(&mut self) {
        perform(self);
    }
}
