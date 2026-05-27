use crate::ssa::*;

fn is_zero(func: &FunctionDef, v: ValueId) -> bool {
    func.get_iconst(v) == Some(0)
}

fn is_one(func: &FunctionDef, v: ValueId) -> bool {
    func.get_iconst(v) == Some(1)
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
                let zero = func.make_iconst(inst.parent, func.get_type(a), 0);
                return replace(func, inst, id, zero);
            }
        }

        InstKind::Mul => {
            let (a, b) = bin(inst);

            if is_zero(func, a) || is_zero(func, b) {
                let zero = func.make_iconst(inst.parent, func.get_type(a), 0);
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
                let zero = func.make_iconst(inst.parent, func.get_type(a), 0);
                return replace(func, inst, id, zero);
            }

            if a == b {
                return replace(func, inst, id, a);
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

pub fn algebraic_simplify(m: &mut Module, f: FuncId) -> bool {
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
