use crate::ir::*;

pub fn dce(func: &mut FunctionDef) {
    let mut worklist: Vec<ValueId> = func
        .values
        .iter()
        .filter(|(_, v)| v.uses.is_empty())
        .map(|(&id, _)| id)
        .collect();

    while let Some(v) = worklist.pop() {
        if let Some(val) = func.values.get(&v) {
            let inst_id = val.def;
            if inst_id == InstId::MAX {
                continue;
            }

            let inst = &func.insts[&inst_id];
            if inst.kind.has_side_effects() {
                continue;
            }

            let ops = inst.operands.clone();
            func.remove_inst(inst_id);

            for op in ops {
                if func.values[&op].uses.is_empty() {
                    worklist.push(op);
                }
            }
        }
    }
}
