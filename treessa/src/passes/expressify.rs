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

        Expr::Const(_) => {}

        Expr::Add(a, b)
        | Expr::Sub(a, b)
        | Expr::Mul(a, b)
        | Expr::Div(a, b)
        | Expr::BitShl(a, b)
        | Expr::BitShr(a, b)
        | Expr::ArithShr(a, b) => {
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

    matches!(func.get_expr(v), Expr::Const(_)) || matches!(uses.get(&v), Some(&1))
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

        Expr::Const(_) => expr,

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
        Expr::Div(a, b) => {
            let a = expand_expr(func, *a, cache, uses, params, changed);
            let b = expand_expr(func, *b, cache, uses, params, changed);
            Expr::Div(Box::new(a), Box::new(b))
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
    }
}
