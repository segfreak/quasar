use std::collections::{HashMap, HashSet};

use crate::analysis::dom::Dominance;
use crate::ir::*;
use quasar::*;

/// canonical key for GVN
#[derive(Hash, Clone, PartialEq, Eq)]
struct GvnKey {
    kind: InstKind,
    args: Vec<(VN, Type)>,
    ty: Type,
}

/// value number instead of raw ValueId
#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug)]
struct VN(u32);

#[derive(Default)]
struct GvnCtx {
    next_vn: u32,
    value_vn: HashMap<ValueId, VN>,
    table: HashMap<GvnKey, ValueId>,
}

impl GvnCtx {
    fn new() -> Self {
        Self {
            next_vn: 1,
            value_vn: HashMap::new(),
            table: HashMap::new(),
        }
    }

    fn vn_of(&mut self, v: ValueId) -> VN {
        *self.value_vn.entry(v).or_insert_with(|| {
            let vn = VN(self.next_vn);
            self.next_vn += 1;
            vn
        })
    }
}

/// canonicalize commutative ops
fn canonicalize(kind: &InstKind, mut ops: Vec<ValueId>) -> Vec<ValueId> {
    match kind {
        InstKind::Add | InstKind::Mul | InstKind::And | InstKind::Or | InstKind::Xor => {
            ops.sort_unstable();
        }
        _ => {}
    }
    ops
}

/// build key safely
fn make_key(ctx: &mut GvnCtx, func: &FunctionDef, inst: &Inst) -> Option<GvnKey> {
    if inst.kind.has_side_effects() || inst.kind.is_alloca() {
        return None;
    }

    let result_ty = inst.result.map(|v| func.get_type(v))?;

    let ops = canonicalize(&inst.kind, inst.operands.clone());

    let args = ops
        .into_iter()
        .map(|v| {
            let ty = func.get_type(v);
            let vn = ctx.vn_of(v);
            (vn, ty)
        })
        .collect();

    Some(GvnKey {
        kind: inst.kind.clone(),
        ty: result_ty,
        args,
    })
}

/// collect replacements first (IMPORTANT FIX)
fn collect_gvn(
    func: &mut FunctionDef,
    dom: &Dominance,
    block: BlockId,
    ctx: &mut GvnCtx,
    repl: &mut HashMap<ValueId, ValueId>,
    visited: &mut HashSet<BlockId>,
) {
    if !visited.insert(block) {
        return;
    }

    let mut local_table = ctx.table.clone();

    let insts = func.blocks[&block].insts.clone();

    for inst_id in insts {
        if !func.insts.contains_key(&inst_id) {
            continue;
        }

        let inst = func.insts[&inst_id].clone();

        let Some(result) = inst.result else {
            continue;
        };

        let Some(key) = make_key(ctx, func, &inst) else {
            continue;
        };

        if let Some(&existing) = local_table.get(&key) {
            if existing != result {
                log::trace!(
                    "replacing %{}:{} (B{}) -> %{}:{} (B{})",
                    result,
                    func.get_type(result),
                    func.get_def_block(result).unwrap(),
                    existing,
                    func.get_type(existing),
                    func.get_def_block(existing).unwrap(),
                );
                repl.insert(result, existing);
            }
        } else {
            local_table.insert(key, result);
        }

        ctx.vn_of(result);
    }

    // recurse dom tree
    for (&child, &parent) in &dom.idom {
        if parent == block {
            let saved = ctx.table.clone();
            ctx.table = local_table.clone();

            collect_gvn(func, dom, child, ctx, repl, visited);

            ctx.table = saved;
        }
    }
}

/// apply replacements safely
fn apply_replacements(func: &mut FunctionDef, repl: &HashMap<ValueId, ValueId>) -> bool {
    if repl.is_empty() {
        return false;
    }

    let mut changed = false;

    // update instructions
    for inst in func.insts.values_mut() {
        for op in &mut inst.operands {
            if let Some(&to) = repl.get(op) {
                *op = to;
                changed = true;
            }
        }
    }

    // update values (uses)
    for v in func.values.values_mut() {
        v.uses.clear();
    }

    // rebuild uses
    for (iid, inst) in &func.insts {
        for (i, &op) in inst.operands.iter().enumerate() {
            if let Some(v) = func.values.get_mut(&op) {
                v.uses.push(Use {
                    inst: *iid,
                    index: i as u32,
                });
            }
        }
    }

    changed
}

/// full pass
pub fn gvn(module: &mut Module, f: FuncId) -> bool {
    let func = module
        .get_function_mut(f)
        .unwrap()
        .get_definition_mut()
        .unwrap();

    let dom = Dominance::build(func);

    let entry = func.entry;

    let mut ctx = GvnCtx::new();
    let mut repl = HashMap::new();
    let mut visited = HashSet::new();

    collect_gvn(func, &dom, entry, &mut ctx, &mut repl, &mut visited);

    apply_replacements(func, &repl)
}
