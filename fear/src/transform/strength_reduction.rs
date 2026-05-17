use crate::ir::*;

use fearcore::*;

fn is_power_of_two(func: &FunctionDef, v: ValueId) -> Option<u32> {
    let c = func.get_iconst(v)?;

    if c <= 0 {
        return None;
    }

    if (c & (c - 1)) != 0 {
        return None;
    }

    Some(c.trailing_zeros())
}

fn normalize_commutative(a: ValueId, b: ValueId, func: &FunctionDef) -> (ValueId, ValueId) {
    let a_is_const = func.get_iconst(a).is_some();
    let b_is_const = func.get_iconst(b).is_some();

    if a_is_const && !b_is_const {
        (b, a)
    } else {
        (a, b)
    }
}

fn try_reduce_inst(func: &mut FunctionDef, id: InstId) -> bool {
    let inst = match func.insts.get(&id).cloned() {
        Some(i) => i,
        None => return false,
    };

    match inst.kind {
        InstKind::Mul => {
            let (mut a, mut b) = (inst.operands[0], inst.operands[1]);

            (a, b) = normalize_commutative(a, b, func);

            if let Some(k) = is_power_of_two(func, b) {
                let shift = func.make_iconst(inst.parent, Type::I32, k as i64);

                let new_inst = Inst {
                    kind: InstKind::LShl,
                    operands: vec![a, shift],
                    parent: inst.parent,
                    result: inst.result,
                };

                return func.replace_inst(id, new_inst);
            }
        }

        InstKind::Div { signed } if !signed => {
            let (a, b) = (inst.operands[0], inst.operands[1]);

            if let Some(k) = is_power_of_two(func, b) {
                let shift = func.make_iconst(inst.parent, Type::I32, k as i64);

                let new_inst = Inst {
                    kind: InstKind::LShr,
                    operands: vec![a, shift],
                    parent: inst.parent,
                    result: inst.result,
                };

                return func.replace_inst(id, new_inst);
            }
        }

        _ => {}
    }

    false
}

pub fn strength_reduction(m: &mut Module, f: FuncId) -> bool {
    let func = m.get_function_mut(f).unwrap().get_definition_mut().unwrap();

    let mut changed = false;

    let inst_ids: Vec<InstId> = func.insts.keys().copied().collect();

    for id in inst_ids {
        if !func.insts.contains_key(&id) {
            continue;
        }

        if try_reduce_inst(func, id) {
            changed = true;
        }
    }

    changed
}
