use crate::{ir::*, types::Type};

fn is_zero(func: &FunctionDef, v: ValueId) -> bool {
    func.get_iconst(v) == Some(0)
}

fn is_one(func: &FunctionDef, v: ValueId) -> bool {
    func.get_iconst(v) == Some(1)
}

fn try_simplify(func: &mut FunctionDef, id: InstId) -> bool {
    let inst = match func.insts.get(&id).cloned() {
        Some(i) => i,
        None => return false,
    };

    match inst.kind {
        InstKind::Add => {
            let (a, b) = (inst.operands[0], inst.operands[1]);

            // x + 0 => x
            if is_zero(func, a) {
                func.replace_value(inst.result.unwrap(), b);
                func.remove_inst(id);
                return true;
            }

            if is_zero(func, b) {
                func.replace_value(inst.result.unwrap(), a);
                func.remove_inst(id);
                return true;
            }

            // x + x => x << 1
            // if a == b {
            //     let shift = func.make_iconst(inst.parent, Type::I32, 1);
            //     let new_inst = Inst {
            //         kind: InstKind::LShl,
            //         operands: vec![a, shift],
            //         parent: inst.parent,
            //         result: inst.result,
            //     };
            //     func.replace_inst(id, new_inst);
            //     return true;
            // }
        }

        InstKind::Sub => {
            let (a, b) = (inst.operands[0], inst.operands[1]);

            // x - 0 => x
            if is_zero(func, b) {
                func.replace_value(inst.result.unwrap(), a);
                func.remove_inst(id);
                return true;
            }

            // x - x => 0
            if a == b {
                let zero = func.make_iconst(inst.parent, Type::I32, 0);
                func.replace_value(inst.result.unwrap(), zero);
                func.remove_inst(id);
                return true;
            }
        }

        InstKind::Mul => {
            let (a, b) = (inst.operands[0], inst.operands[1]);

            // x * 0 => 0
            if is_zero(func, a) || is_zero(func, b) {
                let zero = func.make_iconst(inst.parent, Type::I32, 0);
                func.replace_value(inst.result.unwrap(), zero);
                func.remove_inst(id);
                return true;
            }

            // x * 1 => x
            if is_one(func, a) {
                func.replace_value(inst.result.unwrap(), b);
                func.remove_inst(id);
                return true;
            }

            if is_one(func, b) {
                func.replace_value(inst.result.unwrap(), a);
                func.remove_inst(id);
                return true;
            }
        }

        InstKind::And => {
            let (a, b) = (inst.operands[0], inst.operands[1]);

            // x & 0 => 0
            if is_zero(func, a) || is_zero(func, b) {
                let zero = func.make_iconst(inst.parent, Type::I32, 0);
                func.replace_value(inst.result.unwrap(), zero);
                func.remove_inst(id);
                return true;
            }

            // x & x => x
            if a == b {
                func.replace_value(inst.result.unwrap(), a);
                func.remove_inst(id);
                return true;
            }
        }

        _ => {}
    }

    false
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
