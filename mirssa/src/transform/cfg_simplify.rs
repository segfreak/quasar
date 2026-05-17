use std::collections::VecDeque;

use crate::ir::*;
use quasar::*;

pub fn cfg_simplify(m: &mut Module, fid: FuncId) -> bool {
    let func = match m.get_function_mut(fid).and_then(|f| f.get_definition_mut()) {
        Some(f) => f,
        None => return false,
    };

    let mut changed = false;

    changed |= fold_const_branches(func);
    changed |= remove_unreachable(func);

    changed
}

pub fn remove_unreachable(f: &mut FunctionDef) -> bool {
    let mut visited = HashSet::new();
    let mut q = VecDeque::new();

    q.push_back(f.entry);

    while let Some(b) = q.pop_front() {
        if !visited.insert(b) {
            continue;
        }

        if let Some(block) = f.blocks.get(&b) {
            for &s in &block.succs {
                q.push_back(s);
            }
        }
    }

    let unreachable: Vec<BlockId> = f
        .blocks
        .keys()
        .copied()
        .filter(|b| !visited.contains(b))
        .collect();

    let changed = !unreachable.is_empty();

    for b in unreachable {
        f.remove_block(b);
    }

    changed
}

fn fold_const_branches(f: &mut FunctionDef) -> bool {
    let mut rewrites = Vec::new();

    for (id, inst) in &f.insts {
        if let InstKind::JumpIf {
            then_block,
            else_block,
        } = inst.kind
        {
            let (cond, then_params, else_params) = f.get_jumpif_params(inst).unwrap();
            if let Some(v) = f.try_get_iconst(cond) {
                let target = if v != 0 {
                    (then_block, then_params.to_vec())
                } else {
                    (else_block, else_params.to_vec())
                };
                rewrites.push((*id, target));
            }
        }
    }

    for (id, target) in &rewrites {
        let block = f.insts.get(id).unwrap().parent;
        f.remove_inst(*id);
        f.make_jump(block, target.0, target.1.clone());
    }

    let changed = !rewrites.is_empty();

    if changed {
        f.recompute_control_flow();
    }

    changed
}
