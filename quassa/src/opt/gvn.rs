use crate::analysis::dom::Dominance;
use crate::ir::*;
use quasar::*;

#[derive(Hash, Clone, PartialEq, Eq)]
struct GvnKey {
    kind: InstKind,
    args: Vec<ValueId>,
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

/// value numbering context
#[derive(Clone)]
struct GvnCtx {
    value_number: HashMap<ValueId, u32>,
    next_vn: u32,
    table: HashMap<GvnKey, ValueId>,
}

impl GvnCtx {
    fn new() -> Self {
        Self {
            value_number: HashMap::new(),
            next_vn: 1,
            table: HashMap::new(),
        }
    }

    fn get_vn(&mut self, v: ValueId) -> u32 {
        *self.value_number.entry(v).or_insert_with(|| {
            let id = self.next_vn;
            self.next_vn += 1;
            id
        })
    }

    fn make_key(&mut self, inst: &Inst) -> Option<GvnKey> {
        if inst.kind.has_side_effects() || inst.kind.is_alloca() {
            return None;
        }

        let mut args = inst.operands.clone();

        args = canonicalize(&inst.kind, args);

        let args_vn = args.into_iter().map(|v| self.get_vn(v)).collect();

        Some(GvnKey {
            kind: inst.kind.clone(),
            args: args_vn,
        })
    }
}

/// DFS over dominator tree
fn gvn_block(
    func: &mut FunctionDef,
    dom: &Dominance,
    block: BlockId,
    ctx: &mut GvnCtx,
    changed: &mut bool,
) {
    let mut local_table = ctx.table.clone();

    let inst_ids = func.blocks[&block].insts.clone();

    for inst_id in inst_ids {
        if !func.insts.contains_key(&inst_id) {
            continue;
        }

        let inst = func.insts[&inst_id].clone();

        let result = match inst.result {
            Some(v) => v,
            None => continue,
        };

        let key = match ctx.make_key(&inst) {
            Some(k) => k,
            None => continue,
        };

        if let Some(&existing) = local_table.get(&key) {
            if existing != result {
                log::trace!(
                    "replacing %{} (B{}) -> %{} (B{})",
                    result,
                    func.get_def_block(result).unwrap(),
                    existing,
                    func.get_def_block(existing).unwrap(),
                );
                func.replace_value(result, existing);
                func.remove_inst(inst_id);
                *changed = true;
            }
        } else {
            local_table.insert(key, result);
        }

        ctx.get_vn(result);
    }

    let children: Vec<BlockId> = dom
        .idom
        .iter()
        .filter(|(_, p)| **p == block)
        .map(|(&b, _)| b)
        .collect();

    for child in children {
        let saved = ctx.table.clone();

        ctx.table = local_table.clone();
        gvn_block(func, dom, child, ctx, changed);
        ctx.table = saved;
    }
}

pub fn gvn(module: &mut Module, f: FuncId) -> bool {
    let func = module
        .get_function_mut(f)
        .unwrap()
        .get_definition_mut()
        .unwrap();
    let dom = Dominance::build(func);

    let entry = func.entry;

    let mut ctx = GvnCtx::new();
    let mut changed = false;

    gvn_block(func, &dom, entry, &mut ctx, &mut changed);

    changed
}
