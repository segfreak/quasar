use crate::ssa::*;

fn can_eliminate(inst: &Inst) -> bool {
    !inst.kind.has_side_effects() || matches!(inst.kind, InstKind::Load { volatile: false })
}

pub fn dce(m: &mut Module, f: FuncId) -> bool {
    let func = m.get_function_mut(f).unwrap().get_definition_mut().unwrap();

    let mut changed = false;

    let mut worklist: Vec<ValueId> = func
        .get_values()
        .iter()
        .filter(|(_, v)| v.get_uses().is_empty())
        .map(|(&id, _)| id)
        .collect();

    while let Some(v) = worklist.pop() {
        let val = match func.get_value(v) {
            Some(v) => v,
            None => continue,
        };

        let inst_id = val.get_def();
        if inst_id == InstId::MAX {
            continue;
        }

        let inst = match func.get_inst(inst_id) {
            Some(i) => i.clone(),
            None => continue,
        };

        if !can_eliminate(&inst) {
            continue;
        }

        let ops = inst.operands.clone();

        func.remove_inst(inst_id);
        changed = true;

        for op in ops {
            #[allow(clippy::collapsible_if)]
            if let Some(v) = func.get_value(op) {
                if v.get_uses().is_empty() {
                    worklist.push(op);
                }
            }
        }
    }

    changed
}
