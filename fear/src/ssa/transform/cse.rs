use std::collections::HashMap;

use crate::ssa::*;
use crate::types::Type;

fn canonical_ops(kind: &InstKind, mut ops: Vec<(ValueId, Type)>) -> Vec<(ValueId, Type)> {
    match kind {
        InstKind::Add | InstKind::Mul | InstKind::And | InstKind::Or | InstKind::Xor => {
            ops.sort_unstable_by_key(|(v, _)| *v);
        }
        _ => {}
    }
    ops
}

#[derive(Hash, Clone, PartialEq, Eq)]
struct InstKey {
    kind: InstKind,
    ops: Vec<(ValueId, Type)>,
    ty: Type, // result type
}

pub fn cse(m: &mut Module, f: FuncId) -> bool {
    let func = m.get_function_mut(f).unwrap().get_definition_mut().unwrap();

    let mut changed = false;

    let mut table: HashMap<(BlockId, InstKey), ValueId> = HashMap::new();

    let inst_ids = func.get_inst_ids();
    for id in inst_ids {
        let inst = match func.get_inst(id).cloned() {
            Some(i) => i,
            None => continue,
        };

        let result = match inst.result {
            Some(v) => v,
            None => continue,
        };

        if inst.kind.has_side_effects() || inst.kind.is_alloca() {
            continue;
        }

        let ty = func.get_type_of(result);

        let ops: Vec<(ValueId, Type)> = inst
            .operands
            .iter()
            .map(|&v| (v, func.get_type_of(v)))
            .collect();

        let key = InstKey {
            kind: inst.kind.clone(),
            ops: canonical_ops(&inst.kind, ops),
            ty,
        };

        if let Some(&existing) = table.get(&(inst.parent, key.clone())) {
            if existing != result {
                log::trace!(
                    "replacing %{}:{} (B{}) -> %{}:{} (B{})",
                    result,
                    func.get_type_of(result),
                    func.get_value_def_in(result).unwrap(),
                    existing,
                    func.get_type_of(existing),
                    func.get_value_def_in(existing).unwrap(),
                );
                func.replace_uses(result, existing);
                func.remove_inst(id);
                changed = true;
            }
        } else {
            table.insert((inst.parent, key), result);
        }
    }

    changed
}
