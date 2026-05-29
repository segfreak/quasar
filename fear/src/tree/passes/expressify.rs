use crate::tree::*;

use std::collections::{HashMap, HashSet};

fn build_use_counts(func: &FunctionDef) -> HashMap<ValueId, usize> {
    let mut uses = HashMap::new();

    for v in func.values.values() {
        collect_uses(v, &mut uses);
    }

    uses
}

fn collect_uses(expr: &Expr, uses: &mut HashMap<ValueId, usize>) {
    match &expr.kind {
        ExprKind::Var(v) => {
            *uses.entry(*v).or_insert(0) += 1;
        }

        ExprKind::Const(_)
        | ExprKind::FConst(_)
        | ExprKind::Alloca(_)
        | ExprKind::NAlloca(_, _) => {}

        ExprKind::Square(a)
        | ExprKind::FSquare(a)
        | ExprKind::Cast(_, a)
        | ExprKind::BitNeg(a)
        | ExprKind::Load(_, a) => {
            collect_uses(a, uses);
        }

        ExprKind::Call(_, params) => {
            for expr in params {
                collect_uses(expr, uses);
            }
        }

        ExprKind::Add(a, b)
        | ExprKind::Sub(a, b)
        | ExprKind::Mul(a, b)
        | ExprKind::Div(_, a, b)
        | ExprKind::Rem(_, a, b)
        | ExprKind::FAdd(a, b)
        | ExprKind::FSub(a, b)
        | ExprKind::FMul(a, b)
        | ExprKind::FDiv(a, b)
        | ExprKind::FRem(a, b)
        | ExprKind::BitShl(a, b)
        | ExprKind::BitShr(a, b)
        | ExprKind::ArithShr(a, b)
        | ExprKind::BitAnd(a, b)
        | ExprKind::BitOr(a, b)
        | ExprKind::BitXor(a, b)
        | ExprKind::Cmp(_, a, b)
        | ExprKind::FCmp(_, a, b)
        | ExprKind::Store(_, a, b)
        | ExprKind::PtrOffset(a, b)
        | ExprKind::ElementPtr(_, a, b) => {
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
        *func.values.get_mut(&vid).unwrap() = expanded;
    }

    {
        use crate::tree::passes::dce;
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

    matches!(func.get_expr(v).kind, ExprKind::Const(_))
        || matches!(uses.get(&v), Some(&1))
            && !matches!(
                func.get_expr(v).kind,
                ExprKind::Alloca(_)
                    | ExprKind::Load(_, _)
                    | ExprKind::Store(_, _, _)
                    | ExprKind::Call(_, _)
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

    let expr = func.values.get(&vid).unwrap();
    let result = expand_expr(func, expr.clone(), cache, uses, params, changed);
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
    let kind = match expr.kind {
        ExprKind::Const(_) | ExprKind::FConst(_) => expr.kind,

        ExprKind::Var(v) => {
            if should_inline(func, v, uses, params) {
                *changed = true;
                expand_value(func, v, cache, uses, params, changed).kind
            } else {
                ExprKind::Var(v)
            }
        }

        ExprKind::Alloca(ty) => ExprKind::Alloca(ty),
        ExprKind::NAlloca(ty, cnt) => ExprKind::NAlloca(ty, cnt),
        ExprKind::Call(func, params) => ExprKind::Call(func, params),

        ExprKind::Cast(kind, a) => {
            let a = expand_expr(func, *a, cache, uses, params, changed);
            ExprKind::Cast(kind, Box::new(a))
        }
        ExprKind::PtrOffset(a, b) => {
            let a = expand_expr(func, *a, cache, uses, params, changed);
            let b = expand_expr(func, *b, cache, uses, params, changed);
            ExprKind::PtrOffset(Box::new(a), Box::new(b))
        }
        ExprKind::ElementPtr(ty, a, b) => {
            let a = expand_expr(func, *a, cache, uses, params, changed);
            let b = expand_expr(func, *b, cache, uses, params, changed);
            ExprKind::ElementPtr(ty, Box::new(a), Box::new(b))
        }

        ExprKind::Load(volatile, ptr) => {
            let ptr = expand_expr(func, *ptr, cache, uses, params, changed);
            ExprKind::Load(volatile, Box::new(ptr))
        }
        ExprKind::Store(volatile, ptr, value) => {
            let ptr = expand_expr(func, *ptr, cache, uses, params, changed);
            // let value = expand_expr(func, *value, cache, uses, params, changed);
            ExprKind::Store(volatile, Box::new(ptr), value.clone())
        }

        ExprKind::Cmp(kind, a, b) => {
            let a = expand_expr(func, *a, cache, uses, params, changed);
            let b = expand_expr(func, *b, cache, uses, params, changed);
            ExprKind::Cmp(kind, Box::new(a), Box::new(b))
        }

        ExprKind::FCmp(kind, a, b) => {
            let a = expand_expr(func, *a, cache, uses, params, changed);
            let b = expand_expr(func, *b, cache, uses, params, changed);
            ExprKind::FCmp(kind, Box::new(a), Box::new(b))
        }

        ExprKind::Add(a, b) => {
            let a = expand_expr(func, *a, cache, uses, params, changed);
            let b = expand_expr(func, *b, cache, uses, params, changed);
            ExprKind::Add(Box::new(a), Box::new(b))
        }
        ExprKind::Sub(a, b) => {
            let a = expand_expr(func, *a, cache, uses, params, changed);
            let b = expand_expr(func, *b, cache, uses, params, changed);
            ExprKind::Sub(Box::new(a), Box::new(b))
        }
        ExprKind::Mul(a, b) => {
            let a = expand_expr(func, *a, cache, uses, params, changed);
            let b = expand_expr(func, *b, cache, uses, params, changed);
            ExprKind::Mul(Box::new(a), Box::new(b))
        }
        ExprKind::Div(signed, a, b) => {
            let a = expand_expr(func, *a, cache, uses, params, changed);
            let b = expand_expr(func, *b, cache, uses, params, changed);
            ExprKind::Div(signed, Box::new(a), Box::new(b))
        }
        ExprKind::Rem(signed, a, b) => {
            let a = expand_expr(func, *a, cache, uses, params, changed);
            let b = expand_expr(func, *b, cache, uses, params, changed);
            ExprKind::Rem(signed, Box::new(a), Box::new(b))
        }
        ExprKind::Square(a) => {
            let a = expand_expr(func, *a, cache, uses, params, changed);
            ExprKind::Square(Box::new(a))
        }

        ExprKind::FAdd(a, b) => {
            let a = expand_expr(func, *a, cache, uses, params, changed);
            let b = expand_expr(func, *b, cache, uses, params, changed);
            ExprKind::FAdd(Box::new(a), Box::new(b))
        }
        ExprKind::FSub(a, b) => {
            let a = expand_expr(func, *a, cache, uses, params, changed);
            let b = expand_expr(func, *b, cache, uses, params, changed);
            ExprKind::FSub(Box::new(a), Box::new(b))
        }
        ExprKind::FMul(a, b) => {
            let a = expand_expr(func, *a, cache, uses, params, changed);
            let b = expand_expr(func, *b, cache, uses, params, changed);
            ExprKind::FMul(Box::new(a), Box::new(b))
        }
        ExprKind::FDiv(a, b) => {
            let a = expand_expr(func, *a, cache, uses, params, changed);
            let b = expand_expr(func, *b, cache, uses, params, changed);
            ExprKind::FDiv(Box::new(a), Box::new(b))
        }
        ExprKind::FRem(a, b) => {
            let a = expand_expr(func, *a, cache, uses, params, changed);
            let b = expand_expr(func, *b, cache, uses, params, changed);
            ExprKind::FRem(Box::new(a), Box::new(b))
        }
        ExprKind::FSquare(a) => {
            let a = expand_expr(func, *a, cache, uses, params, changed);
            ExprKind::FSquare(Box::new(a))
        }

        ExprKind::BitShl(a, b) => {
            let a = expand_expr(func, *a, cache, uses, params, changed);
            let b = expand_expr(func, *b, cache, uses, params, changed);
            ExprKind::BitShl(Box::new(a), Box::new(b))
        }
        ExprKind::BitShr(a, b) => {
            let a = expand_expr(func, *a, cache, uses, params, changed);
            let b = expand_expr(func, *b, cache, uses, params, changed);
            ExprKind::BitShr(Box::new(a), Box::new(b))
        }
        ExprKind::ArithShr(a, b) => {
            let a = expand_expr(func, *a, cache, uses, params, changed);
            let b = expand_expr(func, *b, cache, uses, params, changed);
            ExprKind::ArithShr(Box::new(a), Box::new(b))
        }
        ExprKind::BitAnd(a, b) => {
            let a = expand_expr(func, *a, cache, uses, params, changed);
            let b = expand_expr(func, *b, cache, uses, params, changed);
            ExprKind::BitAnd(Box::new(a), Box::new(b))
        }
        ExprKind::BitOr(a, b) => {
            let a = expand_expr(func, *a, cache, uses, params, changed);
            let b = expand_expr(func, *b, cache, uses, params, changed);
            ExprKind::BitOr(Box::new(a), Box::new(b))
        }
        ExprKind::BitXor(a, b) => {
            let a = expand_expr(func, *a, cache, uses, params, changed);
            let b = expand_expr(func, *b, cache, uses, params, changed);
            ExprKind::BitXor(Box::new(a), Box::new(b))
        }
        ExprKind::BitNeg(a) => {
            let a = expand_expr(func, *a, cache, uses, params, changed);
            ExprKind::BitNeg(Box::new(a))
        }
    };

    Expr { ty: expr.ty, kind }
}
