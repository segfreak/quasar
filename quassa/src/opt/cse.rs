use std::collections::HashMap;

use crate::ir::*;

#[derive(Hash, PartialEq, Eq)]
struct InstKey {
    kind: InstKind,
    ops: Vec<ValueId>,
}

pub fn cse(func: &mut FunctionDef) {
    let mut table: HashMap<InstKey, ValueId> = HashMap::new();

    let insts: Vec<InstId> = func.insts.keys().cloned().collect();

    for id in insts {
        if !func.insts.contains_key(&id) {
            continue;
        }

        let inst = &func.insts[&id];
        let result = match inst.result {
            Some(v) => v,
            None => continue,
        };

        if inst.kind.has_side_effects() {
            continue;
        }

        let key = InstKey {
            kind: inst.kind.clone(),
            ops: inst.operands.clone(),
        };

        if let Some(&existing) = table.get(&key) {
            func.replace_value(result, existing);
            func.remove_inst(id);
        } else {
            table.insert(key, result);
        }
    }
}
