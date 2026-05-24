use crate::*;
use std::collections::{HashMap, HashSet};

fn build_use_counts(func: &FunctionDef) -> HashMap<ValueId, usize> {
    let mut uses = HashMap::new();

    for v in func.values.values() {
        collect_uses(&v.expr, &mut uses);
    }

    uses
}

fn collect_uses(expr: &Expr, uses: &mut HashMap<ValueId, usize>) {
    match expr {
        Expr::Var(v) => {
            *uses.entry(*v).or_insert(0) += 1;
        }

        Expr::Const(_) | Expr::Alloca(_) => {}

        Expr::BitNeg(a) | Expr::Load(_, a) => {
            collect_uses(a, uses);
        }

        Expr::Add(a, b)
        | Expr::Sub(a, b)
        | Expr::Mul(a, b)
        | Expr::Div(_, a, b)
        | Expr::BitShl(a, b)
        | Expr::BitShr(a, b)
        | Expr::ArithShr(a, b)
        | Expr::BitAnd(a, b)
        | Expr::BitOr(a, b)
        | Expr::BitXor(a, b)
        | Expr::Cmp(_, a, b)
        | Expr::FCmp(_, a, b)
        | Expr::Store(_, a, b)
        | Expr::PtrOffset(a, b)
        | Expr::ElementPtr(_, a, b) => {
            collect_uses(a, uses);
            collect_uses(b, uses);
        }
    }
}

pub fn expressify(func: &mut FunctionDef) -> bool {
    let mut param_set: HashSet<ValueId> = HashSet::new();
    let mut ordered: Vec<ValueId> = Vec::new();

    let mut block_ids: Vec<BlockId> = func.blocks.keys().copied().collect();
    block_ids.sort();

    for bid in &block_ids {
        for &p in &func.blocks[bid].params {
            param_set.insert(p);
            ordered.push(p);
        }
    }

    for bid in &block_ids {
        for &v in &func.blocks[bid].values {
            ordered.push(v);
        }
    }

    let uses = build_use_counts(func);
    let mut cache: HashMap<ValueId, Expr> = HashMap::new();
    let mut changed = false;

    for vid in ordered {
        let expanded = expand_value(func, vid, &mut cache, &uses, &param_set, &mut changed);
        func.values.get_mut(&vid).unwrap().expr = expanded;
    }

    {
        use crate::passes::dce;
        if changed {
            dce::dce(func);
        }
    }

    changed
}

fn should_inline(
    func: &FunctionDef,
    v: ValueId,
    uses: &HashMap<ValueId, usize>,
    params: &HashSet<ValueId>,
) -> bool {
    if params.contains(&v) {
        return false;
    }

    matches!(func.get_expr(v), Expr::Const(_))
        || matches!(uses.get(&v), Some(&1))
            && !matches!(
                func.get_expr(v),
                Expr::Alloca(_) | Expr::Load(_, _) | Expr::Store(_, _, _)
            )
}

fn expand_value(
    func: &FunctionDef,
    vid: ValueId,
    cache: &mut HashMap<ValueId, Expr>,
    uses: &HashMap<ValueId, usize>,
    params: &HashSet<ValueId>,
    changed: &mut bool,
) -> Expr {
    if let Some(e) = cache.get(&vid) {
        return e.clone();
    }

    let expr = func.values[&vid].expr.clone();
    let result = expand_expr(func, expr, cache, uses, params, changed);

    cache.insert(vid, result.clone());
    result
}

fn expand_expr(
    func: &FunctionDef,
    expr: Expr,
    cache: &mut HashMap<ValueId, Expr>,
    uses: &HashMap<ValueId, usize>,
    params: &HashSet<ValueId>,
    changed: &mut bool,
) -> Expr {
    match expr {
        Expr::Var(v) => {
            if should_inline(func, v, uses, params) {
                *changed = true;
                expand_value(func, v, cache, uses, params, changed)
            } else {
                Expr::Var(v)
            }
        }

        Expr::Alloca(ty) => Expr::Alloca(ty),

        Expr::PtrOffset(a, b) => {
            let a = expand_expr(func, *a, cache, uses, params, changed);
            let b = expand_expr(func, *b, cache, uses, params, changed);
            Expr::PtrOffset(Box::new(a), Box::new(b))
        }
        Expr::ElementPtr(ty, a, b) => {
            let a = expand_expr(func, *a, cache, uses, params, changed);
            let b = expand_expr(func, *b, cache, uses, params, changed);
            Expr::ElementPtr(ty, Box::new(a), Box::new(b))
        }

        Expr::Load(volatile, ptr) => {
            let ptr = expand_expr(func, *ptr, cache, uses, params, changed);
            Expr::Load(volatile, Box::new(ptr))
        }
        Expr::Store(volatile, ptr, value) => {
            let ptr = expand_expr(func, *ptr, cache, uses, params, changed);
            let value = expand_expr(func, *value, cache, uses, params, changed);
            Expr::Store(volatile, Box::new(ptr), Box::new(value))
        }

        Expr::Const(_) => expr,

        Expr::Cmp(kind, a, b) => {
            let a = expand_expr(func, *a, cache, uses, params, changed);
            let b = expand_expr(func, *b, cache, uses, params, changed);
            Expr::Cmp(kind, Box::new(a), Box::new(b))
        }

        Expr::FCmp(kind, a, b) => {
            let a = expand_expr(func, *a, cache, uses, params, changed);
            let b = expand_expr(func, *b, cache, uses, params, changed);
            Expr::FCmp(kind, Box::new(a), Box::new(b))
        }

        Expr::Add(a, b) => {
            let a = expand_expr(func, *a, cache, uses, params, changed);
            let b = expand_expr(func, *b, cache, uses, params, changed);
            Expr::Add(Box::new(a), Box::new(b))
        }
        Expr::Sub(a, b) => {
            let a = expand_expr(func, *a, cache, uses, params, changed);
            let b = expand_expr(func, *b, cache, uses, params, changed);
            Expr::Sub(Box::new(a), Box::new(b))
        }
        Expr::Mul(a, b) => {
            let a = expand_expr(func, *a, cache, uses, params, changed);
            let b = expand_expr(func, *b, cache, uses, params, changed);
            Expr::Mul(Box::new(a), Box::new(b))
        }
        Expr::Div(signed, a, b) => {
            let a = expand_expr(func, *a, cache, uses, params, changed);
            let b = expand_expr(func, *b, cache, uses, params, changed);
            Expr::Div(signed, Box::new(a), Box::new(b))
        }
        Expr::BitShl(a, b) => {
            let a = expand_expr(func, *a, cache, uses, params, changed);
            let b = expand_expr(func, *b, cache, uses, params, changed);
            Expr::BitShl(Box::new(a), Box::new(b))
        }
        Expr::BitShr(a, b) => {
            let a = expand_expr(func, *a, cache, uses, params, changed);
            let b = expand_expr(func, *b, cache, uses, params, changed);
            Expr::BitShr(Box::new(a), Box::new(b))
        }
        Expr::ArithShr(a, b) => {
            let a = expand_expr(func, *a, cache, uses, params, changed);
            let b = expand_expr(func, *b, cache, uses, params, changed);
            Expr::ArithShr(Box::new(a), Box::new(b))
        }
        Expr::BitAnd(a, b) => {
            let a = expand_expr(func, *a, cache, uses, params, changed);
            let b = expand_expr(func, *b, cache, uses, params, changed);
            Expr::BitAnd(Box::new(a), Box::new(b))
        }
        Expr::BitOr(a, b) => {
            let a = expand_expr(func, *a, cache, uses, params, changed);
            let b = expand_expr(func, *b, cache, uses, params, changed);
            Expr::BitOr(Box::new(a), Box::new(b))
        }
        Expr::BitXor(a, b) => {
            let a = expand_expr(func, *a, cache, uses, params, changed);
            let b = expand_expr(func, *b, cache, uses, params, changed);
            Expr::BitXor(Box::new(a), Box::new(b))
        }
        Expr::BitNeg(a) => {
            let a = expand_expr(func, *a, cache, uses, params, changed);
            Expr::BitNeg(Box::new(a))
        }
    }
}
