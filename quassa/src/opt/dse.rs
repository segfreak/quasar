use crate::ir::*;

use std::collections::HashMap;

pub fn dse(func: &mut FunctionDef) {
    let mut last_store: HashMap<ValueId, InstId> = HashMap::new();

    let insts: Vec<InstId> = func.insts.keys().cloned().collect();

    for id in insts {
        if !func.insts.contains_key(&id) {
            continue;
        }

        let inst = &func.insts[&id];

        if let InstKind::Store = inst.kind {
            let mem = inst.operands[0];

            if let Some(prev) = last_store.insert(mem, id) {
                func.remove_inst(prev);
            }
        }
    }
}
