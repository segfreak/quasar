use crate::{ssa::*, types::CastKind};

fn is_copy(inst: &Inst) -> bool {
    matches!(inst.kind, InstKind::Cast(CastKind::Bitcast))
}

pub fn copyprop(m: &mut Module, f: FuncId) -> bool {
    let func = m.get_function_mut(f).unwrap().get_definition_mut().unwrap();

    let mut changed = false;

    let inst_ids = func.get_inst_ids();
    for id in inst_ids {
        let inst = match func.get_inst(id).cloned() {
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
            changed = true;
            continue;
        }

        let uses = func.get_uses(dst).to_vec();

        if uses.is_empty() {
            func.remove_inst(id);
            changed = true;
            continue;
        }

        for u in uses {
            if let Some(user_inst) = func.get_inst_mut(u.inst) {
                user_inst.operands[u.index as usize] = src;
            }

            func.add_use(src, u.inst, u.index);
        }

        func.clear_uses(dst);

        func.remove_inst(id);
        changed = true;
    }

    changed
}
