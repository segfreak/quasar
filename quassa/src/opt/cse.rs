use crate::ir::*;
use quasar::*;

fn canonical_ops(kind: InstKind, mut ops: Vec<ValueId>) -> Vec<ValueId> {
    match kind {
        InstKind::Add | InstKind::Mul | InstKind::And | InstKind::Or | InstKind::Xor => {
            ops.sort_unstable();
        }
        _ => {}
    }
    ops
}

#[derive(Hash, Clone, PartialEq, Eq)]
struct InstKey {
    kind: InstKind,
    ops: Vec<ValueId>,
}

pub fn cse(m: &mut Module, f: FuncId) -> bool {
    let func = m.get_function_mut(f).unwrap().get_definition_mut().unwrap();

    let mut changed = false;

    let mut table: HashMap<(BlockId, InstKey), ValueId> = HashMap::new();
    let insts: Vec<InstId> = func.insts.keys().cloned().collect();

    for id in insts {
        if !func.insts.contains_key(&id) {
            continue;
        }

        let inst = &func.insts[&id];

        let result = match inst.result {
            Some(v) => v,
            None => continue,
        };

        if inst.kind.has_side_effects() || inst.kind.is_alloca() {
            continue;
        }

        let key = InstKey {
            kind: inst.kind.clone(),
            ops: canonical_ops(inst.kind.clone(), inst.operands.clone()),
        };

        if let Some(&existing) = table.get(&(inst.parent, key.clone())) {
            if existing != result {
                log::trace!(
                    "replacing %{} (B{}) -> %{} (B{})",
                    result,
                    func.get_def_block(result).unwrap(),
                    existing,
                    func.get_def_block(existing).unwrap(),
                );
                func.replace_value(result, existing);
                func.remove_inst(id);
                changed = true;
            }
        } else {
            table.insert((inst.parent, key), result);
        }
    }

    changed
}
