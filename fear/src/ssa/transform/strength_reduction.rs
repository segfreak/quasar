use crate::ssa::*;

fn is_power_of_two(func: &FunctionDef, v: ValueId) -> Option<u32> {
    let c = func.get_int_const(v)?;

    if c <= 0 {
        return None;
    }

    if (c & (c - 1)) != 0 {
        return None;
    }

    Some(c.trailing_zeros())
}

fn normalize_commutative(a: ValueId, b: ValueId, func: &FunctionDef) -> (ValueId, ValueId) {
    let a_is_const = func.get_int_const(a).is_some();
    let b_is_const = func.get_int_const(b).is_some();

    if a_is_const && !b_is_const {
        (b, a)
    } else {
        (a, b)
    }
}

fn try_reduce_inst(func: &mut FunctionDef, id: InstId) -> bool {
    let inst = match func.get_inst(id).cloned() {
        Some(i) => i,
        None => return false,
    };

    match inst.kind {
        InstKind::Mul => {
            let (mut a, mut b) = (inst.operands[0], inst.operands[1]);
            (a, b) = normalize_commutative(a, b, func);
            let ty = func.get_type_of(b);

            if let Some(k) = is_power_of_two(func, b) {
                let shift = func.make_int_const(inst.parent, ty, k as i64);

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
            let ty = func.get_type_of(b);
            if let Some(k) = is_power_of_two(func, b) {
                let shift = func.make_int_const(inst.parent, ty, k as i64);

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

    let inst_ids = func.get_inst_ids();
    for id in inst_ids {
        if !func.get_insts().contains_key(&id) {
            continue;
        }

        if try_reduce_inst(func, id) {
            changed = true;
        }
    }

    changed
}
