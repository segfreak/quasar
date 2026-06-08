use std::collections::{hash_map::Entry, HashMap, HashSet, VecDeque};

use crate::{
    ssa::{analysis::dom, *},
    types::Type,
};

pub fn mem2reg(m: &mut Module, f: FuncId) -> bool {
    let func = m.get_function_mut(f).unwrap().get_definition_mut().unwrap();

    let promotable = collect_promotable(func);
    if promotable.is_empty() {
        return false;
    }

    let rpo = func.compute_rpo();
    if rpo.is_empty() {
        return false;
    }

    let idom = dom::compute_idom(func);
    let df = dom::compute_df(func, &idom);

    let mut changed = false;

    for alloca_val in promotable {
        let alloca_inst = match func.get_value_def(alloca_val) {
            Some(id) => id,
            None => continue,
        };

        let alloca_ty = match func.insts.get(&alloca_inst).map(|i| i.kind.clone()) {
            Some(InstKind::Alloca(ty)) => ty,
            _ => continue,
        };

        let def_blocks = store_blocks(func, alloca_val);
        let phi_blocks = insert_phis(func, alloca_ty, &def_blocks, &df);

        rename(func, alloca_val, alloca_ty, &rpo, &idom, &phi_blocks);
        func.remove_inst(alloca_inst);

        changed = true;
    }

    if changed {
        func.reconstruct();
    }

    changed
}

fn collect_promotable(func: &FunctionDef) -> Vec<ValueId> {
    let mut out = Vec::new();

    for inst in func.insts.values() {
        if !inst.kind.is_alloca() {
            continue;
        }

        let result = match inst.result {
            Some(v) => v,
            None => continue,
        };

        let val = match func.values.get(&result) {
            Some(v) => v,
            None => continue,
        };

        let promotable = val.uses.iter().all(|u| {
            let using_inst = match func.insts.get(&u.inst) {
                Some(i) => i,
                None => return false,
            };

            match &using_inst.kind {
                InstKind::Load { volatile: false } => u.index == 0,
                InstKind::Store { volatile: false } => u.index == 1,
                _ => false,
            }
        });

        if promotable {
            out.push(result);
        }
    }

    out
}

fn insert_phis(
    func: &mut FunctionDef,
    ty: Type,
    def_blocks: &HashSet<BlockId>,
    df: &HashMap<BlockId, HashSet<BlockId>>,
) -> HashMap<BlockId, ValueId> {
    let mut phi_blocks: HashMap<BlockId, ValueId> = HashMap::new();
    let mut worklist: VecDeque<BlockId> = def_blocks.iter().copied().collect();
    let mut visited: HashSet<BlockId> = def_blocks.clone();

    while let Some(b) = worklist.pop_front() {
        for &frontier in df.get(&b).into_iter().flatten() {
            if let Entry::Vacant(e) = phi_blocks.entry(frontier) {
                let phi_val = func.add_block_param(frontier, ty);
                e.insert(phi_val);

                if visited.insert(frontier) {
                    worklist.push_back(frontier);
                }
            }
        }
    }

    phi_blocks
}

fn rename(
    func: &mut FunctionDef,
    alloca_val: ValueId,
    ty: Type,
    rpo: &[BlockId],
    idom: &HashMap<BlockId, BlockId>,
    phi_blocks: &HashMap<BlockId, ValueId>,
) {
    let entry = func.entry;
    if rpo.is_empty() {
        return;
    }

    let undef = materialize_undef(func, ty);

    let mut dt_children: HashMap<BlockId, Vec<BlockId>> = HashMap::new();
    for &b in rpo {
        dt_children.entry(b).or_default();
        #[allow(clippy::collapsible_if)]
        if let Some(&parent) = idom.get(&b) {
            if parent != b {
                dt_children.entry(parent).or_default().push(b);
            }
        }
    }

    rename_block(
        func,
        entry,
        alloca_val,
        undef,
        phi_blocks,
        &dt_children,
        &mut Vec::new(),
    );
}

fn rename_block(
    func: &mut FunctionDef,
    block_id: BlockId,
    alloca_val: ValueId,
    undef: ValueId,
    phi_blocks: &HashMap<BlockId, ValueId>,
    dt_children: &HashMap<BlockId, Vec<BlockId>>,
    stack: &mut Vec<ValueId>,
) {
    let checkpoint = stack.len();

    if let Some(&phi_val) = phi_blocks.get(&block_id) {
        stack.push(phi_val);
    }

    let mut current = stack.last().copied().unwrap_or(undef);

    let insts: Vec<InstId> = func
        .try_get_block(block_id)
        .map(|b| b.insts.clone())
        .unwrap_or_default();

    let mut to_remove: Vec<InstId> = Vec::new();

    for inst_id in insts {
        let inst = match func.insts.get(&inst_id) {
            Some(i) => i.clone(),
            None => continue,
        };

        match &inst.kind {
            InstKind::Load { volatile: false } if inst.operands.first() == Some(&alloca_val) => {
                let result = match inst.result {
                    Some(v) => v,
                    None => continue,
                };

                func.replace_uses(result, current);
                to_remove.push(inst_id);
            }

            InstKind::Store { volatile: false } if inst.operands.first() == Some(&alloca_val) => {
                let stored = match inst.operands.get(1) {
                    Some(v) => *v,
                    None => continue,
                };

                current = stored;
                stack.push(stored);
                to_remove.push(inst_id);
            }

            _ => {}
        }
    }

    for id in to_remove {
        func.remove_inst(id);
    }

    let outgoing = stack.last().copied().unwrap_or(undef);

    let succs: Vec<BlockId> = func
        .blocks
        .get(&block_id)
        .map(|b| b.succs.clone())
        .unwrap_or_default();

    for succ in succs {
        if phi_blocks.contains_key(&succ) {
            append_edge_value(func, block_id, outgoing);
        }
    }

    if let Some(children) = dt_children.get(&block_id) {
        for &child in children {
            rename_block(
                func,
                child,
                alloca_val,
                undef,
                phi_blocks,
                dt_children,
                stack,
            );
        }
    }

    stack.truncate(checkpoint);
}

fn append_edge_value(func: &mut FunctionDef, pred: BlockId, value: ValueId) {
    let term_id = match func.blocks.get(&pred).and_then(|b| b.term) {
        Some(id) => id,
        None => return,
    };

    if let Some(term) = func.insts.get_mut(&term_id) {
        term.operands.push(value);
    }

    func.add_use(value, term_id, {
        let term = func.insts.get(&term_id).unwrap();
        (term.operands.len() - 1) as u32
    });
}

fn materialize_undef(func: &mut FunctionDef, ty: Type) -> ValueId {
    func.make_undef(func.get_entry(), ty)
}

fn store_blocks(func: &FunctionDef, alloca_val: ValueId) -> HashSet<BlockId> {
    let mut out = HashSet::new();

    let uses = func
        .values
        .get(&alloca_val)
        .map(|v| v.uses.clone())
        .unwrap_or_default();

    for u in uses {
        let inst = match func.insts.get(&u.inst) {
            Some(i) => i,
            None => continue,
        };

        if matches!(inst.kind, InstKind::Store { volatile: false }) {
            out.insert(inst.parent);
        }
    }

    out
}
