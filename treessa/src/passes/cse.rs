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

        Expr::Add(a, b) | Expr::Sub(a, b) | Expr::Mul(a, b) => {
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
