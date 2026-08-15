use crate::{
    ssa::{transform::constfold::eval_int_cmp, *},
    types::Type,
};

/// Trivial compares folding
pub fn tcf(m: &mut Module, f: FuncId) -> bool {
    use crate::types::IntCmp;

    let func = m.get_function_mut(f).unwrap().get_definition_mut().unwrap();
    let mut changed = false;

    let inst_ids = func.get_inst_ids();
    for id in inst_ids {
        let inst = match func.get_inst(id).cloned() {
            Some(i) => i,
            None => continue,
        };

        let cmp_kind = match &inst.kind {
            InstKind::Cmp(k) => *k,
            _ => continue,
        };

        let lhs = inst.operands[0];
        let rhs = inst.operands[1];
        let result = match inst.result {
            Some(r) => r,
            None => continue,
        };

        let folded: Option<i64> = if lhs == rhs {
            Some(match cmp_kind {
                IntCmp::Eq | IntCmp::ULe | IntCmp::Le | IntCmp::UGe | IntCmp::Ge => 1,
                IntCmp::Ne | IntCmp::ULt | IntCmp::Lt | IntCmp::UGt | IntCmp::Gt => 0,
            })
        } else {
            match (func.get_int_const(lhs), func.get_int_const(rhs)) {
                (Some(a), Some(b)) => Some(eval_int_cmp(cmp_kind, a, b) as i64),
                _ => None,
            }
        };

        if let Some(val) = folded {
            let parent = inst.parent;
            let new_val = func.make_int_const(parent, Type::Int1, val);
            func.replace_uses(result, new_val);
            func.remove_inst(id);
            changed = true;
        }
    }

    changed
}
