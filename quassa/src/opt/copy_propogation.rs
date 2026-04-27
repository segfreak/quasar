use crate::ir::*;

fn is_copy(inst: &Inst) -> bool {
    matches!(inst.kind, InstKind::Cast(CastKind::Bitcast))
}

pub fn copy_propogation(func: &mut FunctionDef) {
    let inst_ids: Vec<InstId> = func.insts.keys().copied().collect();

    for id in inst_ids {
        let inst = match func.insts.get(&id).cloned() {
            Some(i) => i,
            None => continue,
        };

        if !is_copy(&inst) {
            continue;
        }

        if inst.operands.len() != 1 {
            continue;
        }

        let src = inst.operands[0];
        let dst = match inst.result {
            Some(v) => v,
            None => continue,
        };

        if src == dst {
            func.remove_inst(id);
            continue;
        }

        let uses = match func.values.get(&dst) {
            Some(v) => v.uses.clone(),
            None => continue,
        };

        for u in uses {
            if let Some(user_inst) = func.insts.get_mut(&u.inst) {
                user_inst.operands[u.index as usize] = src;
            }

            func.add_use(src, u.inst, u.index);
        }

        if let Some(v) = func.values.get_mut(&dst) {
            v.uses.clear();
        }

        func.remove_inst(id);
    }
}
