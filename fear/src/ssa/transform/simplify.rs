use crate::{ssa::*, types::Type};

fn is_zero(func: &FunctionDef, v: ValueId) -> bool {
    func.get_int_const(v) == Some(0)
}

fn is_one(func: &FunctionDef, v: ValueId) -> bool {
    func.get_int_const(v) == Some(1)
}

fn is_all_ones(func: &FunctionDef, v: ValueId) -> bool {
    let ty = func.get_type_of(v);

    let expected = match ty {
        Type::Int8 => u8::MAX as u64,
        Type::Int16 => u16::MAX as u64,
        Type::Int32 => u32::MAX as u64,
        Type::Int64 => u64::MAX,
        _ => {
            return false;
        }
    };

    if let Some(val) = func.get_int_const(v) {
        val as u64 == expected
    } else {
        false
    }
}

pub fn try_simplify(func: &mut FunctionDef, id: InstId) -> bool {
    let inst = match func.insts.get(&id) {
        Some(i) => &i.clone(),
        None => return false,
    };

    let bin = |i: &Inst| (i.operands[0], i.operands[1]);

    match inst.kind {
        InstKind::Add => {
            let (a, b) = bin(inst);

            if is_zero(func, a) {
                return replace(func, inst, id, b);
            }
            if is_zero(func, b) {
                return replace(func, inst, id, a);
            }
        }

        InstKind::Sub => {
            let (a, b) = bin(inst);

            if is_zero(func, b) {
                return replace(func, inst, id, a);
            }

            if a == b {
                let zero = func.make_int_const(inst.parent, func.get_type_of(a), 0);
                return replace(func, inst, id, zero);
            }
        }

        InstKind::Mul => {
            let (a, b) = bin(inst);

            if is_zero(func, a) || is_zero(func, b) {
                let zero = func.make_int_const(inst.parent, func.get_type_of(a), 0);
                return replace(func, inst, id, zero);
            }

            if is_one(func, a) {
                return replace(func, inst, id, b);
            }

            if is_one(func, b) {
                return replace(func, inst, id, a);
            }
        }

        InstKind::And => {
            let (a, b) = bin(inst);

            if is_zero(func, a) || is_zero(func, b) {
                let zero = func.make_int_const(inst.parent, func.get_type_of(a), 0);
                return replace(func, inst, id, zero);
            }

            if is_all_ones(func, a) {
                return replace(func, inst, id, b);
            }
            if is_all_ones(func, b) {
                return replace(func, inst, id, a);
            }

            if a == b {
                return replace(func, inst, id, a);
            }
        }

        InstKind::Or => {
            let (a, b) = bin(inst);

            if is_zero(func, a) {
                return replace(func, inst, id, b);
            }
            if is_zero(func, b) {
                return replace(func, inst, id, a);
            }

            if is_all_ones(func, a) {
                return replace(func, inst, id, a);
            }
            if is_all_ones(func, b) {
                return replace(func, inst, id, b);
            }

            if a == b {
                return replace(func, inst, id, a);
            }
        }

        InstKind::Xor => {
            let (a, b) = bin(inst);

            if is_zero(func, a) {
                return replace(func, inst, id, b);
            }
            if is_zero(func, b) {
                return replace(func, inst, id, a);
            }

            if a == b {
                let zero = func.make_int_const(inst.parent, func.get_type_of(a), 0);
                return replace(func, inst, id, zero);
            }
        }

        _ => {}
    }

    false
}

fn replace(func: &mut FunctionDef, inst: &Inst, id: InstId, val: ValueId) -> bool {
    let res = inst.result.unwrap();
    func.replace_uses(res, val);
    func.remove_inst(id);
    true
}

pub fn simplify(m: &mut Module, f: FuncId) -> bool {
    let func = m.get_function_mut(f).unwrap().get_definition_mut().unwrap();

    let mut changed = false;

    loop {
        let mut local_change = false;

        let inst_ids: Vec<InstId> = func.insts.keys().copied().collect();

        for id in inst_ids {
            if !func.insts.contains_key(&id) {
                continue;
            }

            if try_simplify(func, id) {
                local_change = true;
                changed = true;
            }
        }

        if !local_change {
            break;
        }
    }

    changed
}
