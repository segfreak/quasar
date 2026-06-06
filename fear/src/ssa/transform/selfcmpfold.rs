use crate::{
    ssa::*,
    types::{IntCmp, Type},
};

pub fn selfcmpfold(m: &mut Module, f: FuncId) -> bool {
    let mut changed = false;

    let Some(func) = m.get_function_mut(f) else {
        return false;
    };

    let Some(def) = func.get_definition_mut() else {
        return false;
    };

    let mut replacements: Vec<(InstId, ValueId)> = Vec::new();

    for (inst_id, inst) in def.insts.clone().iter() {
        match &inst.kind {
            InstKind::Cmp(pred) => {
                let [lhs, rhs] = match inst.operands.as_slice() {
                    [a, b] => [*a, *b],
                    _ => continue,
                };

                if lhs == rhs {
                    let val = match pred {
                        IntCmp::Eq | IntCmp::Le | IntCmp::ULe | IntCmp::Ge | IntCmp::UGe => true,
                        IntCmp::Ne | IntCmp::Lt | IntCmp::ULt | IntCmp::Gt | IntCmp::UGt => false,
                    };

                    let v = if val { 1 } else { 0 };
                    let new_val = def.make_iconst(inst.parent, Type::Int1, v);

                    replacements.push((*inst_id, new_val));
                    changed = true;
                }
            }

            _ => {}
        }
    }

    for (inst_id, new_val) in replacements {
        if let Some(inst) = def.insts.get(&inst_id).cloned() {
            if let Some(res) = inst.result {
                def.replace_uses(res, new_val);
                def.remove_inst(inst_id);
            }
        }
    }

    changed
}
