use std::collections::{HashSet, VecDeque};

use crate::ssa::*;

pub fn cfg_simplify(m: &mut Module, f: FuncId) -> bool {
    let mut any_changed = false;

    loop {
        let mut changed = false;
        changed |= fold_constant_branches(m, f);
        changed |= eliminate_dead_blocks(m, f);
        changed |= eliminate_forwarding_blocks(m, f);
        changed |= merge_blocks(m, f);
        changed |= thread_jumps(m, f);
        changed |= eliminate_redundant_phis(m, f);

        if !changed {
            break;
        }
        any_changed = true;
    }

    if any_changed {
        let func = m.get_function_mut(f).unwrap().get_definition_mut().unwrap();
        func.reconstruct();
    }

    any_changed
}

fn fold_constant_branches(m: &mut Module, f: FuncId) -> bool {
    let func = m.get_function_mut(f).unwrap().get_definition_mut().unwrap();
    let mut changed = false;

    let block_ids: Vec<BlockId> = func.blocks.keys().cloned().collect();

    for block_id in block_ids {
        let term_id = match func.blocks.get(&block_id).and_then(|b| b.term) {
            Some(t) => t,
            None => continue,
        };

        let term = match func.insts.get(&term_id) {
            Some(i) => i.clone(),
            None => continue,
        };

        let (cond, then_block, else_block) = match &term.kind {
            InstKind::JumpIf {
                then_block,
                else_block,
            } => (term.operands[0], *then_block, *else_block),
            _ => continue,
        };

        let const_val = match func.get_int_const(cond) {
            Some(v) => v,
            None => continue,
        };

        let t_params = func
            .blocks
            .get(&then_block)
            .map(|b| b.params.len())
            .unwrap_or(0);
        let e_params = func
            .blocks
            .get(&else_block)
            .map(|b| b.params.len())
            .unwrap_or(0);
        let then_args: Vec<ValueId> = term.operands[1..1 + t_params].to_vec();
        let else_args: Vec<ValueId> = term.operands[1 + t_params..1 + t_params + e_params].to_vec();

        let (target, target_args, dead_target) = if const_val != 0 {
            (then_block, then_args, else_block)
        } else {
            (else_block, else_args, then_block)
        };

        if let Some(db) = func.blocks.get_mut(&dead_target) {
            db.preds.retain(|&p| p != block_id);
        }
        if let Some(b) = func.blocks.get_mut(&block_id) {
            b.succs.retain(|&s| s != dead_target);
        }

        let new_term = Inst {
            kind: InstKind::Jump(target),
            operands: target_args,
            parent: block_id,
            result: None,
        };
        func.replace_inst(term_id, new_term);

        if let Some(b) = func.blocks.get_mut(&block_id) {
            b.term = Some(term_id);
        }

        changed = true;
    }

    changed
}

fn eliminate_dead_blocks(m: &mut Module, f: FuncId) -> bool {
    let func = m.get_function_mut(f).unwrap().get_definition_mut().unwrap();

    let reachable = {
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        queue.push_back(func.entry);
        visited.insert(func.entry);
        while let Some(b) = queue.pop_front() {
            for &s in &func
                .blocks
                .get(&b)
                .map(|bl| bl.succs.clone())
                .unwrap_or_default()
            {
                if visited.insert(s) {
                    queue.push_back(s);
                }
            }
        }
        visited
    };

    let dead: Vec<BlockId> = func
        .blocks
        .keys()
        .cloned()
        .filter(|b| !reachable.contains(b))
        .collect();

    if dead.is_empty() {
        return false;
    }

    for b in dead {
        func.remove_block(b);
    }

    true
}

fn eliminate_forwarding_blocks(m: &mut Module, f: FuncId) -> bool {
    let func = m.get_function_mut(f).unwrap().get_definition_mut().unwrap();
    let entry = func.entry;
    let mut changed = false;

    'outer: loop {
        let candidates: Vec<BlockId> = func
            .blocks
            .iter()
            .filter(|(bid, b)| {
                **bid != entry && b.params.is_empty() && is_forwarding_block(func, **bid)
            })
            .map(|(&bid, _)| bid)
            .collect();

        if candidates.is_empty() {
            break 'outer;
        }

        for fwd in candidates {
            let (target, fwd_args) = match forwarding_target(func, fwd) {
                Some(x) => x,
                None => continue,
            };

            if target == fwd {
                continue;
            }

            let preds: Vec<BlockId> = func
                .blocks
                .get(&fwd)
                .map(|b| b.preds.clone())
                .unwrap_or_default();

            for pred in &preds {
                redirect_successor(func, *pred, fwd, target, &fwd_args);
            }

            if let Some(tb) = func.blocks.get_mut(&target) {
                tb.preds.retain(|&p| p != fwd);
                for &p in &preds {
                    if !tb.preds.contains(&p) {
                        tb.preds.push(p);
                    }
                }
            }

            func.remove_block(fwd);
            changed = true;
        }

        if !changed {
            break;
        }
    }

    changed
}

fn is_forwarding_block(func: &FunctionDef, bid: BlockId) -> bool {
    let insts = match func.blocks.get(&bid) {
        Some(b) => &b.insts,
        None => return false,
    };

    if insts.len() != 1 {
        return false;
    }

    matches!(
        func.insts.get(&insts[0]).map(|i| &i.kind),
        Some(InstKind::Jump(_))
    )
}

fn forwarding_target(func: &FunctionDef, bid: BlockId) -> Option<(BlockId, Vec<ValueId>)> {
    let inst_id = func.blocks.get(&bid)?.insts.first().copied()?;
    let inst = func.insts.get(&inst_id)?;
    match inst.kind {
        InstKind::Jump(target) => Some((target, inst.operands.clone())),
        _ => None,
    }
}

fn redirect_successor(
    func: &mut FunctionDef,
    pred: BlockId,
    old: BlockId,
    new: BlockId,
    new_args: &[ValueId],
) {
    let term_id = match func.blocks.get(&pred).and_then(|b| b.term) {
        Some(t) => t,
        None => return,
    };

    let term = match func.insts.get(&term_id).cloned() {
        Some(i) => i,
        None => return,
    };

    let new_term = match &term.kind {
        InstKind::Jump(target) if *target == old => Inst {
            kind: InstKind::Jump(new),
            operands: new_args.to_vec(),
            parent: pred,
            result: None,
        },

        InstKind::JumpIf {
            then_block,
            else_block,
        } => {
            let t_params = func
                .blocks
                .get(then_block)
                .map(|b| b.params.len())
                .unwrap_or(0);
            let e_params = func
                .blocks
                .get(else_block)
                .map(|b| b.params.len())
                .unwrap_or(0);

            let cond = term.operands[0];
            let then_args: Vec<ValueId> = term.operands[1..1 + t_params].to_vec();
            let else_args: Vec<ValueId> =
                term.operands[1 + t_params..1 + t_params + e_params].to_vec();

            let (new_then, new_then_args, new_else, new_else_args) = if *then_block == old {
                (new, new_args.to_vec(), *else_block, else_args)
            } else if *else_block == old {
                (*then_block, then_args, new, new_args.to_vec())
            } else {
                return; // not our edge
            };

            let mut operands = Vec::with_capacity(1 + new_then_args.len() + new_else_args.len());
            operands.push(cond);
            operands.extend_from_slice(&new_then_args);
            operands.extend_from_slice(&new_else_args);

            Inst {
                kind: InstKind::JumpIf {
                    then_block: new_then,
                    else_block: new_else,
                },
                operands,
                parent: pred,
                result: None,
            }
        }

        _ => return,
    };

    func.replace_inst(term_id, new_term);

    if let Some(pb) = func.blocks.get_mut(&pred) {
        pb.succs.retain(|&s| s != old);
        if !pb.succs.contains(&new) {
            pb.succs.push(new);
        }
    }
}

fn merge_blocks(m: &mut Module, f: FuncId) -> bool {
    let func = m.get_function_mut(f).unwrap().get_definition_mut().unwrap();
    let mut changed = false;

    'outer: loop {
        // Find a mergeable pair (B, S).
        let pair = func.blocks.iter().find_map(|(&bid, b)| {
            let term_id = b.term?;
            let term = func.insts.get(&term_id)?;
            let succ = match term.kind {
                InstKind::Jump(s) => s,
                _ => return None,
            };
            if succ == bid {
                return None; // self-loop
            }
            let succ_block = func.blocks.get(&succ)?;
            // S must have exactly one predecessor and no block params.
            if succ_block.preds.len() == 1 && succ_block.params.is_empty() {
                Some((bid, succ, term_id))
            } else {
                None
            }
        });

        let (pred, succ, term_id) = match pair {
            Some(p) => p,
            None => break 'outer,
        };

        // Remove the jump terminator from pred.
        func.remove_inst(term_id);
        if let Some(b) = func.blocks.get_mut(&pred) {
            b.term = None;
            b.succs.clear();
        }

        // Move all instructions from succ into pred.
        let succ_insts: Vec<InstId> = func
            .blocks
            .get(&succ)
            .map(|b| b.insts.clone())
            .unwrap_or_default();
        let succ_term = func.blocks.get(&succ).and_then(|b| b.term);
        let succ_succs: Vec<BlockId> = func
            .blocks
            .get(&succ)
            .map(|b| b.succs.clone())
            .unwrap_or_default();

        for &inst_id in &succ_insts {
            if let Some(inst) = func.insts.get_mut(&inst_id) {
                inst.parent = pred;
            }
            if let Some(b) = func.blocks.get_mut(&pred) {
                b.insts.push(inst_id);
            }
        }

        // Update terminator / succs of pred.
        if let Some(b) = func.blocks.get_mut(&pred) {
            b.term = succ_term;
            b.succs = succ_succs.clone();
        }

        // Rewrite succ_succs: replace succ with pred in their preds lists.
        for &ss in &succ_succs {
            if let Some(sb) = func.blocks.get_mut(&ss) {
                for p in &mut sb.preds {
                    if *p == succ {
                        *p = pred;
                    }
                }
            }
        }

        // Also rewrite any terminator that references succ as a target block id.
        // (JumpIf kind stores BlockId inline.)
        if let Some(b) = func.blocks.get_mut(&pred) {
            b.insts.retain(|_| true); // no-op; just keep ownership
        }
        if let Some(tid) = succ_term {
            let _term = match func.insts.get(&tid).cloned() {
                Some(i) => i,
                None => {
                    func.blocks.remove(&succ);
                    changed = true;
                    continue 'outer;
                }
            };
            // No need to rewrite block ids in the term itself here — the block
            // ids embedded in JumpIf refer to *targets*, not the current block.
        }

        func.blocks.remove(&succ);
        changed = true;
    }

    changed
}

fn thread_jumps(m: &mut Module, f: FuncId) -> bool {
    let func = m.get_function_mut(f).unwrap().get_definition_mut().unwrap();
    let mut changed = false;

    let block_ids: Vec<BlockId> = func.blocks.keys().cloned().collect();

    for pred_id in block_ids {
        let term_id = match func.blocks.get(&pred_id).and_then(|b| b.term) {
            Some(t) => t,
            None => continue,
        };

        let term = match func.insts.get(&term_id).cloned() {
            Some(i) => i,
            None => continue,
        };

        // Only handle JumpIf at the predecessor.
        let (cond, then_block, else_block) = match &term.kind {
            InstKind::JumpIf {
                then_block,
                else_block,
            } => (term.operands[0], *then_block, *else_block),
            _ => continue,
        };

        // For each arm (known_val=1 → then branch, known_val=0 → else branch):
        for &(arm, known_val) in &[(then_block, 1i64), (else_block, 0i64)] {
            // The arm block must have no block params and end with a jmpif on
            // the *same* condition value.
            let arm_block = match func.blocks.get(&arm) {
                Some(b) => b,
                None => continue,
            };

            if !arm_block.params.is_empty() {
                continue;
            }

            let arm_term_id = match arm_block.term {
                Some(t) => t,
                None => continue,
            };

            let arm_term = match func.insts.get(&arm_term_id).cloned() {
                Some(i) => i,
                None => continue,
            };

            // arm must be: jmpif <same cond> B1, B2
            let (arm_cond, arm_then, arm_else) = match &arm_term.kind {
                InstKind::JumpIf {
                    then_block: at,
                    else_block: ae,
                } => (arm_term.operands[0], *at, *ae),
                _ => continue,
            };

            if arm_cond != cond {
                continue;
            }

            // arm has no other instructions except the jmpif (and possibly
            // iconst for cond itself — but cond is defined elsewhere).
            // Check that arm_block's insts are all either the jmpif or
            // pure no-side-effect instructions.
            let arm_insts: Vec<InstId> = func
                .blocks
                .get(&arm)
                .map(|b| b.insts.clone())
                .unwrap_or_default();

            let only_jmpif = arm_insts.iter().all(|&iid| {
                func.insts
                    .get(&iid)
                    .is_some_and(|i| i.kind.is_terminator() || !i.kind.has_side_effects())
            });

            if !only_jmpif {
                continue;
            }

            // We know `cond` is `known_val` when we reach `arm`.
            // So the jmpif in arm resolves to a fixed target.
            let thread_target = if known_val != 0 { arm_then } else { arm_else };

            // Replace arm's jmpif with an unconditional jump to thread_target.
            let t_params = func
                .blocks
                .get(&arm_then)
                .map(|b| b.params.len())
                .unwrap_or(0);
            let e_params = func
                .blocks
                .get(&arm_else)
                .map(|b| b.params.len())
                .unwrap_or(0);

            let thread_args: Vec<ValueId> = if known_val != 0 {
                arm_term.operands[1..1 + t_params].to_vec()
            } else {
                arm_term.operands[1 + t_params..1 + t_params + e_params].to_vec()
            };

            let dead_target = if known_val != 0 { arm_else } else { arm_then };

            // Rewrite arm's terminator.
            let new_arm_term = Inst {
                kind: InstKind::Jump(thread_target),
                operands: thread_args,
                parent: arm,
                result: None,
            };
            func.replace_inst(arm_term_id, new_arm_term);

            if let Some(ab) = func.blocks.get_mut(&arm) {
                ab.succs.retain(|&s| s != dead_target);
                if !ab.succs.contains(&thread_target) {
                    ab.succs.push(thread_target);
                }
                ab.term = Some(arm_term_id);
            }

            if let Some(db) = func.blocks.get_mut(&dead_target) {
                db.preds.retain(|&p| p != arm);
            }

            changed = true;
        }
    }

    changed
}

fn eliminate_redundant_phis(m: &mut Module, f: FuncId) -> bool {
    let func = m.get_function_mut(f).unwrap().get_definition_mut().unwrap();
    let mut changed = false;

    let block_ids: Vec<BlockId> = func.blocks.keys().cloned().collect();

    'outer: for block_id in block_ids {
        let params: Vec<ValueId> = match func.blocks.get(&block_id) {
            Some(b) => b.params.clone(),
            None => continue,
        };

        let preds: Vec<BlockId> = match func.blocks.get(&block_id) {
            Some(b) => b.preds.clone(),
            None => continue,
        };

        if preds.is_empty() {
            continue;
        }

        // For each param index, collect the argument passed by each predecessor.
        for (param_idx, &param_val) in params.iter().enumerate() {
            let mut incoming: Vec<ValueId> = Vec::new();

            for &pred in &preds {
                let term_id = match func.blocks.get(&pred).and_then(|b| b.term) {
                    Some(t) => t,
                    None => continue 'outer,
                };
                let term = match func.insts.get(&term_id) {
                    Some(i) => i,
                    None => continue 'outer,
                };

                let arg = match &term.kind {
                    InstKind::Jump(_) => term.operands.get(param_idx).copied(),
                    InstKind::JumpIf {
                        then_block,
                        else_block,
                    } => {
                        let t_params = func
                            .blocks
                            .get(then_block)
                            .map(|b| b.params.len())
                            .unwrap_or(0);
                        let _e_params = func
                            .blocks
                            .get(else_block)
                            .map(|b| b.params.len())
                            .unwrap_or(0);

                        if *then_block == block_id {
                            term.operands.get(1 + param_idx).copied()
                        } else if *else_block == block_id {
                            term.operands.get(1 + t_params + param_idx).copied()
                        } else {
                            None
                        }
                    }
                    _ => None,
                };

                match arg {
                    Some(v) => incoming.push(v),
                    None => continue 'outer,
                }
            }

            let first = match incoming.first() {
                Some(&v) => v,
                None => continue,
            };

            let all_same = incoming.iter().all(|&v| v == first) && first != param_val;

            if all_same {
                func.replace_uses(param_val, first);
                if let Some(b) = func.blocks.get_mut(&block_id) {
                    b.params.retain(|&p| p != param_val);
                }
                func.values.remove(&param_val);
                changed = true;
            }
        }
    }

    changed
}
