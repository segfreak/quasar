use crate::ssa::*;

pub fn eval_int_cmp(kind: crate::types::IntCmp, a: i64, b: i64) -> bool {
    use crate::types::IntCmp;
    match kind {
        IntCmp::Eq => a == b,
        IntCmp::Ne => a != b,
        IntCmp::Lt => a < b,
        IntCmp::Le => a <= b,
        IntCmp::Gt => a > b,
        IntCmp::Ge => a >= b,
        // Unsigned: reinterpret as u64.
        IntCmp::ULt => (a as u64) < (b as u64),
        IntCmp::ULe => (a as u64) <= (b as u64),
        IntCmp::UGt => (a as u64) > (b as u64),
        IntCmp::UGe => (a as u64) >= (b as u64),
    }
}

pub fn eval_float_cmp(cmp: crate::types::FloatCmp, a: f64, b: f64) -> bool {
    use crate::types::FloatCmp::*;
    match cmp {
        Ord => !a.is_nan() && !b.is_nan(),
        Uno => a.is_nan() || b.is_nan(),
        OEq => a == b,
        ONe => a != b,
        OGt => a > b,
        OGe => a >= b,
        OLt => a < b,
        OLe => a <= b,
        UEq => a.is_nan() || b.is_nan() || a == b,
        UNe => a.is_nan() || b.is_nan() || a != b,
        UGt => a.is_nan() || a > b,
        UGe => a.is_nan() || a >= b,
        ULt => a.is_nan() || a < b,
        ULe => a.is_nan() || a <= b,
    }
}

pub fn constfold(m: &mut Module, f: FuncId) -> bool {
    let func = m.get_function_mut(f).unwrap().get_definition_mut().unwrap();

    let mut changed = false;

    let inst_ids = func.get_inst_ids();
    for id in inst_ids {
        let inst = match func.get_inst(id).cloned() {
            Some(i) => i,
            None => continue,
        };

        let c: Vec<Option<i64>> = inst
            .operands
            .iter()
            .map(|&v| func.get_int_const(v))
            .collect();

        let folded = match inst.kind {
            InstKind::Add => match (c[0], c[1]) {
                (Some(a), Some(b)) => Some(a + b),
                _ => None,
            },
            InstKind::Sub => match (c[0], c[1]) {
                (Some(a), Some(b)) => Some(a - b),
                _ => None,
            },
            InstKind::Mul => match (c[0], c[1]) {
                (Some(a), Some(b)) => Some(a * b),
                _ => None,
            },
            InstKind::Div { signed } => match (c[0], c[1]) {
                (Some(a), Some(b)) if b != 0 => {
                    if signed {
                        Some(a / b)
                    } else {
                        Some(((a as u64) / (b as u64)) as i64)
                    }
                }
                _ => None,
            },
            InstKind::And => match (c[0], c[1]) {
                (Some(a), Some(b)) => Some(a & b),
                _ => None,
            },
            InstKind::Or => match (c[0], c[1]) {
                (Some(a), Some(b)) => Some(a | b),
                _ => None,
            },
            InstKind::Xor => match (c[0], c[1]) {
                (Some(a), Some(b)) => Some(a ^ b),
                _ => None,
            },
            InstKind::LShl => match (c[0], c[1]) {
                (Some(a), Some(b)) => {
                    let ty = func.get_type_of(inst.operands[0]);
                    let bit_width = ty.get_size() * 8;

                    let lhs = a as u64;
                    let rhs = b as u32;

                    if rhs as usize >= bit_width {
                        None
                    } else {
                        Some(((lhs) << rhs) as i64)
                    }
                }
                _ => None,
            },
            InstKind::LShr => match (c[0], c[1]) {
                (Some(a), Some(b)) => {
                    let ty = func.get_type_of(inst.operands[0]);
                    let bit_width = ty.get_size() * 8;

                    let lhs = a as u64;
                    let rhs = b as u32;

                    if rhs as usize >= bit_width {
                        None
                    } else {
                        Some(((lhs) >> rhs) as i64)
                    }
                }
                _ => None,
            },
            InstKind::AShr => match (c[0], c[1]) {
                (Some(a), Some(b)) => {
                    let ty = func.get_type_of(inst.operands[0]);
                    let bit_width = ty.get_size() * 8;
                    if b < 0 {
                        None
                    } else {
                        let rhs = b as u32;
                        if (rhs as usize) >= bit_width {
                            Some(if a < 0 { -1 } else { 0 })
                        } else {
                            #[allow(clippy::unnecessary_cast)]
                            let lhs = a as i64;
                            Some(lhs >> rhs)
                        }
                    }
                }
                _ => None,
            },
            InstKind::Cmp(kind) => match (c[0], c[1]) {
                (Some(a), Some(b)) => Some(eval_int_cmp(kind, a, b) as i64),
                _ => None,
            },
            _ => None,
        };

        // let ty = inst.result.map(|r| func.get_type_of(r)).unwrap_or(Type::I32);
        // if let Some(val) = folded {
        //     let new_const = func.make_iconst(block, ty, val);
        //     func.replace_value(result, new_const);
        //     func.remove_inst(id);
        // }

        if let Some(val) = folded {
            func.remove_inst_uses(id);
            let inst = func.get_inst_mut(id).unwrap();

            // 2. mutate instruction
            inst.kind = InstKind::IConst(val);
            inst.operands.clear();

            changed = true;
        }
    }

    changed
}
