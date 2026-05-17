use std::collections::HashMap;

use crate::analysis::dom::Dominance;
use crate::ir::*;
use quasar::*;

// ---------------------------------------------------------------------------
// Value numbers
// ---------------------------------------------------------------------------

/// Stable identity for a computed value, independent of ValueId.
#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug)]
struct VN(u32);

/// Canonical key used to look up constants/params by *content* rather than
/// by raw ValueId, so two `IConst(42)` instructions share the same VN.
#[derive(Clone, Hash, PartialEq, Eq, Debug)]
enum ValueKey {
    /// Integer constant with its type (size matters for VN equality).
    IConst(i64, Type),
    /// Float constant with its type.
    FConst(u64, Type),
    /// Block parameter — uniquely identified by its ValueId (no folding).
    Param(ValueId),
}

// ---------------------------------------------------------------------------
// GVN expression key
// ---------------------------------------------------------------------------

/// A fully canonicalized expression.  Arguments are expressed as (VN, Type)
/// pairs so that the key is independent of which specific ValueId was used.
#[derive(Debug, Hash, Clone, PartialEq, Eq)]
struct GvnKey {
    kind: InstKind,
    args: Vec<(VN, Type)>,
    ty: Type,
}

// ---------------------------------------------------------------------------
// Context
// ---------------------------------------------------------------------------

struct GvnCtx {
    next_vn: u32,
    /// ValueId → VN (memoized).
    value_vn: HashMap<ValueId, VN>,
    /// Normalized content → VN (shared across all blocks).
    canonical_vn: HashMap<ValueKey, VN>,
    /// Expression table for the *current* dom-tree scope.
    /// Entries are added on the way down and removed on the way back up.
    table: HashMap<GvnKey, ValueId>,
}

impl GvnCtx {
    fn new() -> Self {
        Self {
            next_vn: 1,
            value_vn: HashMap::new(),
            canonical_vn: HashMap::new(),
            table: HashMap::new(),
        }
    }

    fn fresh_vn(&mut self) -> VN {
        let vn = VN(self.next_vn);
        self.next_vn += 1;
        vn
    }

    /// Return the VN for `v`, creating one if needed.
    ///
    /// * Constants → normalized by (value, type).
    /// * Block parameters → normalized by ValueId (no merging across params).
    /// * Everything else → 1-to-1 with ValueId.
    fn vn_of(&mut self, v: ValueId, func: &FunctionDef) -> VN {
        if let Some(&vn) = self.value_vn.get(&v) {
            return vn;
        }

        let val = match func.values.get(&v) {
            Some(v) => v,
            None => {
                // Stale/invalid id — give it a fresh unique VN.
                let vn = self.fresh_vn();
                self.value_vn.insert(v, vn);
                return vn;
            }
        };

        // Is this a block parameter? (def == InstId::MAX)
        let key = if val.def == InstId::MAX {
            ValueKey::Param(v)
        } else {
            // Try to normalize constants by value.
            match func.insts.get(&val.def).map(|i| &i.kind) {
                Some(InstKind::IConst(x)) => ValueKey::IConst(*x, val.ty),
                Some(InstKind::FConst(x)) => ValueKey::FConst(*x, val.ty),
                _ => {
                    // Regular instruction result — 1-to-1.
                    let vn = self.fresh_vn();
                    self.value_vn.insert(v, vn);
                    return vn;
                }
            }
        };

        let vn = if let Some(&existing) = self.canonical_vn.get(&key) {
            existing
        } else {
            let fresh = self.fresh_vn();
            self.canonical_vn.insert(key, fresh);
            fresh
        };

        self.value_vn.insert(v, vn);
        vn
    }

    fn assign_vn(&mut self, v: ValueId, vn: VN) {
        self.value_vn.insert(v, vn);
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Follow the replacement chain to its root.
/// In practice this is at most 1-2 hops because we process in dom order.
fn resolve(mut v: ValueId, repl: &HashMap<ValueId, ValueId>) -> ValueId {
    while let Some(&next) = repl.get(&v) {
        v = next;
    }
    v
}

/// Sort operands of commutative ops so argument order doesn't cause
/// spurious GVN misses.
fn canonicalize(kind: &InstKind, mut ops: Vec<ValueId>) -> Vec<ValueId> {
    match kind {
        InstKind::Add | InstKind::Mul | InstKind::And | InstKind::Or | InstKind::Xor => {
            ops.sort_unstable();
        }
        _ => {}
    }
    ops
}

/// Build the GVN key for `inst`.
///
/// Returns `None` for:
/// * side-effecting instructions (calls, loads, stores)
/// * allocas
/// * instructions without a result
/// * pure constants (IConst/FConst) — handled via `vn_of` directly
fn make_key(
    ctx: &mut GvnCtx,
    func: &FunctionDef,
    inst: &Inst,
    repl: &HashMap<ValueId, ValueId>,
) -> Option<GvnKey> {
    if inst.kind.has_side_effects() || inst.kind.is_alloca() {
        return None;
    }

    // Constants are normalized through ValueKey, not through expression keys.
    if matches!(inst.kind, InstKind::IConst(_) | InstKind::FConst(_)) {
        return None;
    }

    let result = inst.result?;
    let result_ty = func.get_type(result);

    // Apply already-known replacements *before* building the key so that
    // transitively equivalent expressions are recognised.
    let resolved_ops: Vec<ValueId> = inst.operands.iter().map(|&v| resolve(v, repl)).collect();
    let ops = canonicalize(&inst.kind, resolved_ops);

    let args: Vec<(VN, Type)> = ops
        .into_iter()
        .map(|v| {
            let ty = func.get_type(v);
            let vn = ctx.vn_of(v, func);
            (vn, ty)
        })
        .collect();

    Some(GvnKey {
        kind: inst.kind.clone(),
        ty: result_ty,
        args,
    })
}

/// Pre-build a children map so dom-tree traversal is O(n).
fn build_dom_children(dom: &Dominance) -> HashMap<BlockId, Vec<BlockId>> {
    let mut children: HashMap<BlockId, Vec<BlockId>> = HashMap::new();
    for (&child, &parent) in &dom.idom {
        children.entry(parent).or_default().push(child);
    }
    children
}

// ---------------------------------------------------------------------------
// Core dom-tree walk
// ---------------------------------------------------------------------------

/// Walk the dom tree and fill `repl` with redundant-value → canonical-value
/// pairs.
///
/// `scope_keys` acts as a stack: we push keys added in the current subtree
/// and pop them on the way back up — this avoids cloning the entire table
/// at every recursion level.
fn collect_gvn(
    func: &FunctionDef,
    dom_children: &HashMap<BlockId, Vec<BlockId>>,
    block: BlockId,
    ctx: &mut GvnCtx,
    repl: &mut HashMap<ValueId, ValueId>,
    scope_keys: &mut Vec<GvnKey>,
) {
    let scope_start = scope_keys.len();

    let insts = match func.blocks.get(&block) {
        Some(b) => b.insts.clone(),
        None => return,
    };

    for inst_id in insts {
        let inst = match func.insts.get(&inst_id) {
            Some(i) => i.clone(),
            None => continue,
        };

        let result = match inst.result {
            Some(r) => r,
            None => continue,
        };

        // Constants: just ensure they have a VN (normalized by content).
        if matches!(inst.kind, InstKind::IConst(_) | InstKind::FConst(_)) {
            ctx.vn_of(result, func);
            continue;
        }

        match make_key(ctx, func, &inst, repl) {
            None => {
                // Side-effecting or un-hoistable: give it a unique VN so
                // downstream instructions can still reference it.
                let vn = ctx.fresh_vn();
                ctx.assign_vn(result, vn);
            }
            Some(key) => {
                if let Some(&existing) = ctx.table.get(&key) {
                    // Follow the replacement chain on `existing` too, in case
                    // it was itself replaced in an earlier block.
                    let canonical = resolve(existing, repl);
                    if canonical != result {
                        repl.insert(result, canonical);
                        // Share the VN of the canonical value.
                        let vn = ctx.vn_of(canonical, func);
                        ctx.assign_vn(result, vn);
                    }
                } else {
                    let vn = ctx.fresh_vn();
                    ctx.assign_vn(result, vn);
                    ctx.table.insert(key.clone(), result);
                    scope_keys.push(key);
                }
            }
        }
    }

    // Recurse into dominated children.
    if let Some(children) = dom_children.get(&block) {
        for &child in children {
            collect_gvn(func, dom_children, child, ctx, repl, scope_keys);
        }
    }

    // Remove entries that were added in this block's scope.
    // Children already cleaned up their own entries, so scope_keys[scope_start..]
    // contains only keys inserted by *this* block.
    for key in scope_keys.drain(scope_start..) {
        ctx.table.remove(&key);
    }
}

// ---------------------------------------------------------------------------
// Apply replacements
// ---------------------------------------------------------------------------

fn apply_replacements(func: &mut FunctionDef, repl: &HashMap<ValueId, ValueId>) -> bool {
    if repl.is_empty() {
        return false;
    }

    for (&result, &canonical) in repl {
        log::trace!(
            "replacing %{}:{} (B{:?}) -> %{}:{} (B{:?})",
            result,
            func.get_type(result),
            func.get_def_block(result),
            canonical,
            func.get_type(canonical),
            func.get_def_block(canonical),
        );
        func.replace_value(result, canonical);
        if let Some(inst_id) = func.get_def_inst(result) {
            func.remove_inst(inst_id);
        }
    }

    true
}

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

/// Run GVN over function `f`.  Returns `true` if anything changed.
pub fn gvn(module: &mut Module, f: FuncId) -> bool {
    let func = module
        .get_function_mut(f)
        .unwrap()
        .get_definition_mut()
        .unwrap();

    let dom = Dominance::build(func);
    let dom_children = build_dom_children(&dom);
    let entry = func.entry;

    let mut ctx = GvnCtx::new();
    let mut repl: HashMap<ValueId, ValueId> = HashMap::new();
    let mut scope_keys: Vec<GvnKey> = Vec::new();

    collect_gvn(
        func,
        &dom_children,
        entry,
        &mut ctx,
        &mut repl,
        &mut scope_keys,
    );

    apply_replacements(func, &repl)
}
