use std::collections::{HashMap, HashSet};

use crate::ir::*;

pub fn dse(m: &mut Module, f: FuncId) -> bool {
    let def = m.get_function_mut(f).unwrap().get_definition_mut().unwrap();

    let mut changed = false;

    let store_chains = build_store_chains(def);
    let dead_stores = find_dead_stores(def, &store_chains);

    for inst_id in dead_stores {
        def.remove_inst(inst_id);
        changed = true;
    }

    changed
}

fn build_store_chains(def: &FunctionDef) -> HashMap<ValueId, Vec<(InstId, BlockId, usize)>> {
    let mut chains: HashMap<ValueId, Vec<(InstId, BlockId, usize)>> = HashMap::new();

    for (bid, block) in &def.blocks {
        for (pos, inst_id) in block.insts.iter().enumerate() {
            let inst = &def.insts[inst_id];

            if let InstKind::Store { volatile: false } = inst.kind {
                let ptr = inst.operands[0];
                let canonical_addr = canonicalize_address(def, ptr);
                chains
                    .entry(canonical_addr)
                    .or_default()
                    .push((*inst_id, *bid, pos));
            }
        }
    }

    chains
}

fn canonicalize_address(def: &FunctionDef, ptr: ValueId) -> ValueId {
    let val = &def.values[&ptr];

    if val.def == InstId::MAX {
        return ptr;
    }

    let inst = &def.insts[&val.def];

    if inst.kind.is_alloca() {
        return ptr;
    }

    if matches!(inst.kind, InstKind::ElementPtr) {
        return canonicalize_address(def, inst.operands[0]);
    }

    ptr
}

fn find_dead_stores(
    def: &FunctionDef,
    store_chains: &HashMap<ValueId, Vec<(InstId, BlockId, usize)>>,
) -> Vec<InstId> {
    let mut dead = Vec::new();

    for (canonical_addr, stores) in store_chains {
        for (store_id, store_block, store_pos) in stores {
            let has_load_after =
                has_load_after_in_path(def, *store_block, *store_pos, *canonical_addr);

            let may_escape = may_alias_with_external(def, *canonical_addr);

            if !has_load_after && !may_escape {
                dead.push(*store_id);
            }
        }
    }

    dead
}

fn has_load_after_in_path(
    def: &FunctionDef,
    start_block: BlockId,
    start_pos: usize,
    canonical_addr: ValueId,
) -> bool {
    let mut visited = HashSet::new();
    dfs_load_search(def, start_block, start_pos, canonical_addr, &mut visited)
}

fn dfs_load_search(
    def: &FunctionDef,
    block: BlockId,
    start_pos: usize,
    canonical_addr: ValueId,
    visited: &mut HashSet<BlockId>,
) -> bool {
    if visited.contains(&block) {
        return false;
    }
    visited.insert(block);

    let curr_block = &def.blocks[&block];

    #[allow(clippy::collapsible_if)]
    for (pos, &inst_id) in curr_block.insts.iter().enumerate() {
        if pos <= start_pos {
            continue;
        }

        let inst = &def.insts[&inst_id];

        if let InstKind::Load { .. } = inst.kind {
            if may_alias(def, inst.operands[0], canonical_addr) {
                return true;
            }
        }

        if let InstKind::Store { volatile: false } = inst.kind {
            if may_alias(def, inst.operands[0], canonical_addr) {
                return false;
            }
        }

        if let InstKind::Store { volatile: true } = inst.kind {
            if may_alias(def, inst.operands[0], canonical_addr) {
                return true;
            }
        }

        if matches!(inst.kind, InstKind::Call(_)) {
            return true;
        }
    }

    for &succ_block in &curr_block.succs {
        if dfs_load_search(def, succ_block, 0, canonical_addr, visited) {
            return true;
        }
    }

    false
}

fn may_alias(def: &FunctionDef, addr1: ValueId, addr2: ValueId) -> bool {
    if addr1 == addr2 {
        return true;
    }

    let base1 = get_alloca_base(def, addr1);
    let base2 = get_alloca_base(def, addr2);

    match (base1, base2) {
        (Some(b1), Some(b2)) => b1 == b2,
        _ => true,
    }
}

fn get_alloca_base(def: &FunctionDef, addr: ValueId) -> Option<ValueId> {
    let val = &def.values[&addr];

    if val.def == InstId::MAX {
        return None;
    }

    let inst = &def.insts[&val.def];

    if inst.kind.is_alloca() {
        return Some(addr);
    }

    if matches!(inst.kind, InstKind::ElementPtr) {
        return get_alloca_base(def, inst.operands[0]);
    }

    None
}

fn may_alias_with_external(def: &FunctionDef, canonical_addr: ValueId) -> bool {
    let val = &def.values[&canonical_addr];

    if val.def == InstId::MAX {
        return true;
    }

    let inst = &def.insts[&val.def];

    if !inst.kind.is_alloca() {
        return true;
    }

    for u in &val.uses {
        let user_inst = &def.insts[&u.inst];
        if matches!(user_inst.kind, InstKind::Call(_)) {
            return true;
        }
    }

    false
}
