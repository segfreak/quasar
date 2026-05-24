use fear::types::Type;

use crate::{Expr, FunctionDef, ValueId};
use std::collections::HashMap;

pub fn cse(func: &mut FunctionDef) -> bool {
    let mut changed = false;

    let block_ids: Vec<_> = func.blocks.keys().copied().collect();

    for bid in block_ids {
        changed |= cse_block(func, bid);
    }

    changed
}

fn count_expr(expr: &Expr, cnt: &mut HashMap<Expr, usize>) {
    match expr {
        Expr::Var(_) | Expr::Const(_) => {}

        Expr::BitNeg(a) => {
            *cnt.entry(expr.clone()).or_insert(0) += 1;
            count_expr(a, cnt);
        }

        Expr::Alloca(_) => {}
        Expr::Load(_, ptr) => {
            *cnt.entry(expr.clone()).or_insert(0) += 1;
            count_expr(ptr, cnt);
        }

        Expr::Store(_, ptr, value) => {
            *cnt.entry(expr.clone()).or_insert(0) += 1;
            count_expr(ptr, cnt);
            count_expr(value, cnt);
        }

        Expr::PtrOffset(base, offset) => {
            *cnt.entry(expr.clone()).or_insert(0) += 1;
            count_expr(base, cnt);
            count_expr(offset, cnt);
        }

        Expr::ElementPtr(_ty, base, offset) => {
            *cnt.entry(expr.clone()).or_insert(0) += 1;
            count_expr(base, cnt);
            count_expr(offset, cnt);
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
        | Expr::FCmp(_, a, b) => {
            *cnt.entry(expr.clone()).or_insert(0) += 1;

            count_expr(a, cnt);
            count_expr(b, cnt);
        }
    }
}

fn cse_block(func: &mut FunctionDef, bid: u32) -> bool {
    let mut cnt = HashMap::new();
    let mut changed = false;

    let values = func.blocks[&bid].values.clone();

    for v in &values {
        let expr = func.values[v].expr.clone();
        count_expr(&expr, &mut cnt);
    }

    let mut memo: HashMap<Expr, ValueId> = HashMap::new();
    let mut new_values: Vec<ValueId> = Vec::new();

    for v in values {
        let expr = func.values[&v].expr.clone();

        let (new_expr, _) = rewrite(
            func,
            bid,
            expr,
            &mut cnt,
            &mut memo,
            &mut new_values,
            &mut changed,
        );

        func.values.get_mut(&v).unwrap().expr = new_expr;
        new_values.push(v);
    }

    func.blocks.get_mut(&bid).unwrap().values = new_values;

    changed
}

fn rewrite(
    func: &mut FunctionDef,
    bid: u32,
    expr: Expr,
    cnt: &mut HashMap<Expr, usize>,
    memo: &mut HashMap<Expr, ValueId>,
    new_values: &mut Vec<ValueId>,
    changed: &mut bool,
) -> (Expr, Type) {
    match expr {
        Expr::Const(_) => (expr, Type::I32),

        Expr::Var(vid) => {
            let ty = func.values[&vid].ty;
            (Expr::Var(vid), ty)
        }

        Expr::Alloca(ty) => (expr, ty),
        Expr::Load(volatile, ptr) => {
            let (ptr, ty_ptr) = rewrite(func, bid, *ptr, cnt, memo, new_values, changed);
            let expr = Expr::Load(volatile, Box::new(ptr));
            (expr, ty_ptr)
        }
        Expr::Store(volatile, ptr, value) => {
            let (ptr, ty_ptr) = rewrite(func, bid, *ptr, cnt, memo, new_values, changed);
            let (value, _) = rewrite(func, bid, *value, cnt, memo, new_values, changed);
            let expr = Expr::Store(volatile, Box::new(ptr), Box::new(value));
            (expr, ty_ptr)
        }
        Expr::PtrOffset(base, offset) => {
            let (base, ty_base) = rewrite(func, bid, *base, cnt, memo, new_values, changed);
            let (offset, _) = rewrite(func, bid, *offset, cnt, memo, new_values, changed);
            let expr = Expr::PtrOffset(Box::new(base), Box::new(offset));
            hoist_if_needed(func, bid, expr, ty_base, cnt, memo, new_values, changed)
        }
        Expr::ElementPtr(ty, base, offset) => {
            let (base, ty_base) = rewrite(func, bid, *base, cnt, memo, new_values, changed);
            let (offset, _) = rewrite(func, bid, *offset, cnt, memo, new_values, changed);
            let expr = Expr::ElementPtr(ty, Box::new(base), Box::new(offset));
            hoist_if_needed(func, bid, expr, ty_base, cnt, memo, new_values, changed)
        }

        Expr::Add(a, b) => {
            let (a, ty_a) = rewrite(func, bid, *a, cnt, memo, new_values, changed);
            let (b, _) = rewrite(func, bid, *b, cnt, memo, new_values, changed);
            let expr = Expr::Add(Box::new(a), Box::new(b));
            hoist_if_needed(func, bid, expr, ty_a, cnt, memo, new_values, changed)
        }

        Expr::Sub(a, b) => {
            let (a, ty_a) = rewrite(func, bid, *a, cnt, memo, new_values, changed);
            let (b, _) = rewrite(func, bid, *b, cnt, memo, new_values, changed);
            let expr = Expr::Sub(Box::new(a), Box::new(b));
            hoist_if_needed(func, bid, expr, ty_a, cnt, memo, new_values, changed)
        }

        Expr::Mul(a, b) => {
            let (a, ty_a) = rewrite(func, bid, *a, cnt, memo, new_values, changed);
            let (b, _) = rewrite(func, bid, *b, cnt, memo, new_values, changed);
            let expr = Expr::Mul(Box::new(a), Box::new(b));
            hoist_if_needed(func, bid, expr, ty_a, cnt, memo, new_values, changed)
        }

        Expr::BitShl(a, b) => {
            let (a, ty_a) = rewrite(func, bid, *a, cnt, memo, new_values, changed);
            let (b, _) = rewrite(func, bid, *b, cnt, memo, new_values, changed);
            let expr = Expr::BitShl(Box::new(a), Box::new(b));
            hoist_if_needed(func, bid, expr, ty_a, cnt, memo, new_values, changed)
        }

        Expr::BitShr(a, b) => {
            let (a, ty_a) = rewrite(func, bid, *a, cnt, memo, new_values, changed);
            let (b, _) = rewrite(func, bid, *b, cnt, memo, new_values, changed);
            let expr = Expr::BitShr(Box::new(a), Box::new(b));
            hoist_if_needed(func, bid, expr, ty_a, cnt, memo, new_values, changed)
        }

        Expr::ArithShr(a, b) => {
            let (a, ty_a) = rewrite(func, bid, *a, cnt, memo, new_values, changed);
            let (b, _) = rewrite(func, bid, *b, cnt, memo, new_values, changed);
            let expr = Expr::ArithShr(Box::new(a), Box::new(b));
            hoist_if_needed(func, bid, expr, ty_a, cnt, memo, new_values, changed)
        }

        Expr::BitAnd(a, b) => {
            let (a, ty_a) = rewrite(func, bid, *a, cnt, memo, new_values, changed);
            let (b, _) = rewrite(func, bid, *b, cnt, memo, new_values, changed);
            let expr = Expr::BitAnd(Box::new(a), Box::new(b));
            hoist_if_needed(func, bid, expr, ty_a, cnt, memo, new_values, changed)
        }

        Expr::BitOr(a, b) => {
            let (a, ty_a) = rewrite(func, bid, *a, cnt, memo, new_values, changed);
            let (b, _) = rewrite(func, bid, *b, cnt, memo, new_values, changed);
            let expr = Expr::BitOr(Box::new(a), Box::new(b));
            hoist_if_needed(func, bid, expr, ty_a, cnt, memo, new_values, changed)
        }

        Expr::BitXor(a, b) => {
            let (a, ty_a) = rewrite(func, bid, *a, cnt, memo, new_values, changed);
            let (b, _) = rewrite(func, bid, *b, cnt, memo, new_values, changed);
            let expr = Expr::BitXor(Box::new(a), Box::new(b));
            hoist_if_needed(func, bid, expr, ty_a, cnt, memo, new_values, changed)
        }

        Expr::BitNeg(a) => {
            let (a, ty_a) = rewrite(func, bid, *a, cnt, memo, new_values, changed);
            let expr = Expr::BitNeg(Box::new(a));
            hoist_if_needed(func, bid, expr, ty_a, cnt, memo, new_values, changed)
        }

        Expr::Div(signed, a, b) => {
            let (a, ty_a) = rewrite(func, bid, *a, cnt, memo, new_values, changed);
            let (b, _) = rewrite(func, bid, *b, cnt, memo, new_values, changed);
            let expr = Expr::Div(signed, Box::new(a), Box::new(b));
            hoist_if_needed(func, bid, expr, ty_a, cnt, memo, new_values, changed)
        }

        Expr::Cmp(kind, a, b) => {
            let (a, ty_a) = rewrite(func, bid, *a, cnt, memo, new_values, changed);
            let (b, _) = rewrite(func, bid, *b, cnt, memo, new_values, changed);
            let expr = Expr::Cmp(kind, Box::new(a), Box::new(b));
            hoist_if_needed(func, bid, expr, ty_a, cnt, memo, new_values, changed)
        }

        Expr::FCmp(kind, a, b) => {
            let (a, ty_a) = rewrite(func, bid, *a, cnt, memo, new_values, changed);
            let (b, _) = rewrite(func, bid, *b, cnt, memo, new_values, changed);
            let expr = Expr::FCmp(kind, Box::new(a), Box::new(b));
            hoist_if_needed(func, bid, expr, ty_a, cnt, memo, new_values, changed)
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn hoist_if_needed(
    func: &mut FunctionDef,
    _bid: u32,
    expr: Expr,
    ty: Type,
    cnt: &mut HashMap<Expr, usize>,
    memo: &mut HashMap<Expr, ValueId>,
    new_values: &mut Vec<ValueId>,
    changed: &mut bool,
) -> (Expr, Type) {
    if let Some(&vid) = memo.get(&expr) {
        return (Expr::Var(vid), ty);
    }

    let uses = cnt.get(&expr).copied().unwrap_or(0);

    if uses <= 1 {
        return (expr, ty);
    }

    let vid = func.next_value;
    func.next_value += 1;

    func.values.insert(
        vid,
        crate::Value {
            ty,
            expr: expr.clone(),
        },
    );

    new_values.push(vid);
    memo.insert(expr.clone(), vid);

    *changed = true;

    (Expr::Var(vid), ty)
}
