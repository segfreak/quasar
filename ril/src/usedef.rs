use crate::ir::*;
use quasar::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UseDef {
    pub use_set: HashSet<VReg>,
    pub def_set: HashSet<VReg>,
}

pub fn usedef_of_block(block: &Block) -> UseDef {
    let mut use_set = HashSet::new();
    let mut def_set = HashSet::new();

    for inst in &block.insts {
        for v in inst.get_uses() {
            if !def_set.contains(&v) {
                use_set.insert(v);
            }
        }

        if let Some(v) = inst.get_def() {
            def_set.insert(*v);
        }
    }

    UseDef { use_set, def_set }
}
