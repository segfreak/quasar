use std::collections::{HashMap, HashSet};

use crate::ssa::*;

pub fn dse(m: &mut Module, f: FuncId) -> bool {
    global_dse(m, f)
}

fn definitely_not_alias(func: &FunctionDef, a: ValueId, b: ValueId) -> bool {
    if a == b {
        return false;
    }

    let inst_id_a = func.get_value_def(a);
    let inst_id_b = func.get_value_def(b);

    let kind_a = inst_id_a.and_then(|id| func.get_inst(id)).map(|i| &i.kind);
    let kind_b = inst_id_b.and_then(|id| func.get_inst(id)).map(|i| &i.kind);

    match (kind_a, kind_b) {
        (
            Some(InstKind::Alloca(_) | InstKind::NAlloca(_, _)),
            Some(InstKind::Alloca(_) | InstKind::NAlloca(_, _)),
        ) => true,

        (Some(InstKind::Alloca(_) | InstKind::NAlloca(_, _)), None)
        | (None, Some(InstKind::Alloca(_) | InstKind::NAlloca(_, _))) => true,

        (Some(InstKind::ElementPtr(_)), Some(InstKind::ElementPtr(_))) => {
            elementptr_not_alias(func, a, b)
        }

        _ => false,
    }
}

fn elementptr_not_alias(func: &FunctionDef, a: ValueId, b: ValueId) -> bool {
    let id_a = match func.get_value_def(a) {
        Some(i) => i,
        None => return false,
    };
    let id_b = match func.get_value_def(b) {
        Some(i) => i,
        None => return false,
    };

    let inst_a = match func.get_inst(id_a) {
        Some(i) => i,
        None => return false,
    };
    let inst_b = match func.get_inst(id_b) {
        Some(i) => i,
        None => return false,
    };

    if inst_a.operands[0] != inst_b.operands[0] {
        return false;
    }

    match (
        func.get_int_const(inst_a.operands[1]),
        func.get_int_const(inst_b.operands[1]),
    ) {
        (Some(ca), Some(cb)) => ca != cb,
        _ => false,
    }
}

fn is_barrier(kind: &InstKind) -> bool {
    matches!(
        kind,
        InstKind::Store { volatile: true } | InstKind::Load { volatile: true } | InstKind::Call(_)
    )
}

// `None`  = TOP  (all pointers are potentially live – conservative)
// `Some(S)` = only the pointers in S are live
fn union(a: Option<HashSet<ValueId>>, b: Option<HashSet<ValueId>>) -> Option<HashSet<ValueId>> {
    match (a, b) {
        (None, _) | (_, None) => None, // TOP ∪ anything = TOP
        (Some(mut sa), Some(sb)) => {
            sa.extend(sb);
            Some(sa)
        }
    }
}

fn is_ptr_live(func: &FunctionDef, live: &Option<HashSet<ValueId>>, ptr: ValueId) -> bool {
    match live {
        None => true, // conservative: barrier was hit
        Some(s) => s
            .iter()
            .any(|&lp| lp == ptr || !definitely_not_alias(func, lp, ptr)),
    }
}

fn kill(live: &mut Option<HashSet<ValueId>>, ptr: ValueId) {
    if let Some(s) = live {
        s.remove(&ptr);
    }
}

fn global_dse(m: &mut Module, f: FuncId) -> bool {
    let func_ro = m.get_function(f).unwrap().get_definition().unwrap();

    let blocks: Vec<BlockId> = func_ro.compute_rpo();

    let mut live_in: HashMap<BlockId, Option<HashSet<ValueId>>> =
        blocks.iter().map(|&b| (b, Some(HashSet::new()))).collect();

    let mut worklist: Vec<BlockId> = blocks.iter().rev().cloned().collect();

    while let Some(bid) = worklist.pop() {
        let new_live_in = compute_live_in(func_ro, bid, &live_in);

        if new_live_in == live_in[&bid] {
            continue;
        }

        live_in.insert(bid, new_live_in);

        if let Some(block) = func_ro.get_block(bid) {
            for &pred in &block.preds {
                if !worklist.contains(&pred) {
                    worklist.push(pred);
                }
            }
        }
    }

    let func = m.get_function_mut(f).unwrap().get_definition_mut().unwrap();
    let mut changed = false;

    for &block_id in &blocks {
        let insts: Vec<InstId> = match func.get_block(block_id) {
            Some(b) => b.insts.clone(),
            None => continue,
        };

        let succs: Vec<BlockId> = func
            .get_block(block_id)
            .map(|b| b.succs.clone())
            .unwrap_or_default();

        let mut live: Option<HashSet<ValueId>> = Some(HashSet::new());
        for s in succs {
            live = union(
                live,
                live_in.get(&s).cloned().unwrap_or(Some(HashSet::new())),
            );
        }

        let mut to_remove: Vec<InstId> = Vec::new();

        for &inst_id in insts.iter().rev() {
            let inst = match func.get_inst(inst_id) {
                Some(i) => i.clone(),
                None => continue,
            };

            match &inst.kind {
                InstKind::Load { volatile: false } => {
                    let ptr = inst.operands[0];
                    if let Some(ref mut s) = live {
                        s.insert(ptr);
                    }
                }

                InstKind::Store { volatile: false } => {
                    let ptr = inst.operands[0];
                    if !is_ptr_live(func, &live, ptr) {
                        log::trace!("removing dead store {} (ptr {:?})", inst_id, ptr);
                        to_remove.push(inst_id);
                    }
                    kill(&mut live, ptr);
                }

                k if is_barrier(k) => {
                    live = None;
                }

                _ => {}
            }
        }

        for id in to_remove {
            func.remove_inst(id);
            changed = true;
        }
    }

    changed
}

fn compute_live_in(
    func: &FunctionDef,
    block_id: BlockId,
    live_in: &HashMap<BlockId, Option<HashSet<ValueId>>>,
) -> Option<HashSet<ValueId>> {
    let succs = func
        .get_block(block_id)
        .map(|b| b.succs.as_slice())
        .unwrap_or(&[]);

    let mut live: Option<HashSet<ValueId>> = Some(HashSet::new());
    for &s in succs {
        live = union(
            live,
            live_in.get(&s).cloned().unwrap_or(Some(HashSet::new())),
        );
    }

    // walk the block backwards
    let insts = match func.get_block(block_id) {
        Some(b) => b.insts.clone(),
        None => return live,
    };

    for &inst_id in insts.iter().rev() {
        let inst = match func.get_inst(inst_id) {
            Some(i) => i,
            None => continue,
        };

        match &inst.kind {
            InstKind::Load { volatile: false } => {
                let ptr = inst.operands[0];
                if let Some(ref mut s) = live {
                    s.insert(ptr);
                }
            }
            InstKind::Store { volatile: false } => {
                let ptr = inst.operands[0];
                // only kill if we can prove no may-alias in the live set
                //  conservative: only remove the exact pointer
                kill(&mut live, ptr);
            }
            k if is_barrier(k) => {
                live = None;
            }
            _ => {}
        }
    }

    live
}
