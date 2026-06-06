use std::collections::HashMap;

use crate::ssa::analysis::dom::Dominance;
use crate::{ssa::*, types::Type};

#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug)]
struct VN(u32);

#[derive(Clone, Hash, PartialEq, Eq, Debug)]
enum ValueKey {
    IConst(i64, Type),
    FConst(u64, Type),
    Param(ValueId),
}

#[derive(Debug, Hash, Clone, PartialEq, Eq)]
struct GvnKey {
    kind: InstKind,
    args: Vec<(VN, Type)>,
    ty: Type,
}

struct GvnCtx {
    next_vn: u32,
    value_vn: HashMap<ValueId, VN>,
    canonical_vn: HashMap<ValueKey, VN>,
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

    fn vn_of(&mut self, v: ValueId, func: &FunctionDef) -> VN {
        if let Some(&vn) = self.value_vn.get(&v) {
            return vn;
        }

        let val = match func.values.get(&v) {
            Some(v) => v,
            None => {
                let vn = self.fresh_vn();
                self.value_vn.insert(v, vn);
                return vn;
            }
        };

        let key = if val.def == InstId::MAX {
            ValueKey::Param(v)
        } else {
            match func.insts.get(&val.def).map(|i| &i.kind) {
                Some(InstKind::IConst(x)) => ValueKey::IConst(*x, val.ty),
                Some(InstKind::FConst(x)) => ValueKey::FConst(*x, val.ty),
                _ => {
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

fn resolve(mut v: ValueId, repl: &HashMap<ValueId, ValueId>) -> ValueId {
    while let Some(&next) = repl.get(&v) {
        v = next;
    }
    v
}

fn canonicalize(kind: &InstKind, mut ops: Vec<ValueId>) -> Vec<ValueId> {
    match kind {
        InstKind::Add | InstKind::Mul | InstKind::And | InstKind::Or | InstKind::Xor => {
            ops.sort_unstable();
        }
        _ => {}
    }
    ops
}

fn make_key(
    ctx: &mut GvnCtx,
    func: &FunctionDef,
    inst: &Inst,
    repl: &HashMap<ValueId, ValueId>,
) -> Option<GvnKey> {
    if inst.kind.has_side_effects() || inst.kind.is_alloca() {
        return None;
    }

    if matches!(inst.kind, InstKind::IConst(_) | InstKind::FConst(_)) {
        return None;
    }

    let result = inst.result?;
    let result_ty = func.get_type(result);

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

fn build_dom_children(dom: &Dominance) -> HashMap<BlockId, Vec<BlockId>> {
    let mut children: HashMap<BlockId, Vec<BlockId>> = HashMap::new();
    for (&child, &parent) in &dom.idom {
        children.entry(parent).or_default().push(child);
    }
    children
}

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

        if matches!(inst.kind, InstKind::IConst(_) | InstKind::FConst(_)) {
            ctx.vn_of(result, func);
            continue;
        }

        match make_key(ctx, func, &inst, repl) {
            None => {
                let vn = ctx.fresh_vn();
                ctx.assign_vn(result, vn);
            }
            Some(key) => {
                if let Some(&existing) = ctx.table.get(&key) {
                    let canonical = resolve(existing, repl);
                    if canonical != result {
                        repl.insert(result, canonical);
                        let vn = ctx.vn_of(canonical, func);
                        ctx.assign_vn(result, vn);
                    } else {
                        // Same instruction, assign same value number
                        let vn = ctx.vn_of(result, func);
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

    if let Some(children) = dom_children.get(&block) {
        for &child in children {
            collect_gvn(func, dom_children, child, ctx, repl, scope_keys);
        }
    }

    for key in scope_keys.drain(scope_start..) {
        ctx.table.remove(&key);
    }
}

fn apply_replacements(func: &mut FunctionDef, repl: &HashMap<ValueId, ValueId>) -> bool {
    if repl.is_empty() {
        return false;
    }

    for (&result, &canonical) in repl {
        log::trace!(
            "replacing %{}:{} ({:?}) -> %{}:{} ({:?})",
            result,
            func.get_type(result),
            func.get_value_def_in(result).unwrap(),
            canonical,
            func.get_type(canonical),
            func.get_value_def_in(canonical).unwrap(),
        );
        func.replace_uses(result, canonical);
        if let Some(inst_id) = func.get_value_def(result) {
            func.remove_inst(inst_id);
        }
    }

    true
}

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
