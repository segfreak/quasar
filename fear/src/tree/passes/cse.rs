use crate::tree::{Expr, ExprKind, FunctionDef, ValueId};

use std::collections::HashMap;

pub fn cse(m: &crate::ssa::Module, func: &mut FunctionDef) -> bool {
    let mut changed = false;

    let block_ids: Vec<_> = func.blocks.keys().copied().collect();

    for bid in block_ids {
        changed |= cse_block(m, func, bid);
    }

    changed
}

fn count_expr(expr: &Expr, cnt: &mut HashMap<Expr, usize>) {
    // match &expr.kind {
    //     ExprKind::Var(_) | ExprKind::Const(_) | ExprKind::FConst(_) => {}

    //     ExprKind::Call(_, params) => {
    //         *cnt.entry(expr.clone()).or_insert(0) += 1;
    //         for expr in params {
    //             count_expr(expr, cnt);
    //         }
    //     }

    //     ExprKind::BitNeg(a) => {
    //         *cnt.entry(expr.clone()).or_insert(0) += 1;
    //         count_expr(a, cnt);
    //     }

    //     ExprKind::Alloca(_) | ExprKind::NAlloca(_, _) => {}
    //     ExprKind::Load(_, ptr) => {
    //         *cnt.entry(expr.clone()).or_insert(0) += 1;
    //         count_expr(ptr, cnt);
    //     }

    //     ExprKind::Store(_, ptr, value) => {
    //         *cnt.entry(expr.clone()).or_insert(0) += 1;
    //         count_expr(ptr, cnt);
    //         count_expr(value, cnt);
    //     }

    //     ExprKind::PtrOffset(base, offset) => {
    //         *cnt.entry(expr.clone()).or_insert(0) += 1;
    //         count_expr(base, cnt);
    //         count_expr(offset, cnt);
    //     }

    //     ExprKind::ElementPtr(_ty, base, offset) => {
    //         *cnt.entry(expr.clone()).or_insert(0) += 1;
    //         count_expr(base, cnt);
    //         count_expr(offset, cnt);
    //     }

    //     ExprKind::Cast(_kind, a) => {
    //         *cnt.entry(expr.clone()).or_insert(0) += 1;

    //         count_expr(a, cnt);
    //     }

    //     ExprKind::Add(a, b)
    //     | ExprKind::Sub(a, b)
    //     | ExprKind::Mul(a, b)
    //     | ExprKind::Div(_, a, b)
    //     | ExprKind::Rem(_, a, b)
    //     | ExprKind::FAdd(a, b)
    //     | ExprKind::FSub(a, b)
    //     | ExprKind::FMul(a, b)
    //     | ExprKind::FDiv(a, b)
    //     | ExprKind::FRem(a, b)
    //     | ExprKind::BitShl(a, b)
    //     | ExprKind::BitShr(a, b)
    //     | ExprKind::ArithShr(a, b)
    //     | ExprKind::BitAnd(a, b)
    //     | ExprKind::BitOr(a, b)
    //     | ExprKind::BitXor(a, b)
    //     | ExprKind::Cmp(_, a, b)
    //     | ExprKind::FCmp(_, a, b) => {
    //         *cnt.entry(expr.clone()).or_insert(0) += 1;

    //         count_expr(a, cnt);
    //         count_expr(b, cnt);
    //     }
    // }
    if !matches!(
        expr.kind,
        ExprKind::Var(_) | ExprKind::Const(_) | ExprKind::FConst(_)
    ) {
        *cnt.entry(expr.clone()).or_insert(0) += 1;
    }
    for expr in &expr.kind.get_uses() {
        count_expr(expr, cnt);
    }
}

fn cse_block(m: &crate::ssa::Module, func: &mut FunctionDef, bid: u32) -> bool {
    let mut cnt = HashMap::new();
    let mut changed = false;

    let values = func.blocks[&bid].values.clone();

    for v in &values {
        let expr = func.values[v].clone();
        count_expr(&expr, &mut cnt);
    }

    let mut memo: HashMap<Expr, ValueId> = HashMap::new();
    let mut new_values: Vec<ValueId> = Vec::new();

    for v in values {
        let expr = func.values[&v].clone();

        let new_expr = rewrite(
            m,
            func,
            bid,
            expr,
            &mut cnt,
            &mut memo,
            &mut new_values,
            &mut changed,
        );

        *func.values.get_mut(&v).unwrap() = new_expr;
        new_values.push(v);
    }

    func.blocks.get_mut(&bid).unwrap().values = new_values;

    changed
}

#[allow(clippy::too_many_arguments, clippy::only_used_in_recursion)]
fn rewrite(
    m: &crate::ssa::Module,
    func: &mut FunctionDef,
    bid: u32,
    expr: Expr,
    cnt: &mut HashMap<Expr, usize>,
    memo: &mut HashMap<Expr, ValueId>,
    new_values: &mut Vec<ValueId>,
    changed: &mut bool,
) -> Expr {
    let ty = expr.ty;

    match &expr.kind {
        ExprKind::Const(_)
        | ExprKind::FConst(_)
        | ExprKind::Var(_)
        | ExprKind::Alloca(_)
        | ExprKind::NAlloca(_, _) => expr,

        ExprKind::Cast(kind, a) => {
            let a = rewrite(m, func, bid, *a.clone(), cnt, memo, new_values, changed);
            let new_expr = Expr {
                ty,
                kind: ExprKind::Cast(*kind, Box::new(a)),
            };
            hoist_if_needed(func, new_expr, cnt, memo, new_values, changed)
        }

        ExprKind::Call(func_id, params) => {
            let new_params: Vec<Expr> = params
                .iter()
                .map(|p| rewrite(m, func, bid, p.clone(), cnt, memo, new_values, changed))
                .collect();
            Expr {
                ty,
                kind: ExprKind::Call(*func_id, new_params),
            }
        }

        ExprKind::Load(volatile, ptr) => {
            let ptr = rewrite(m, func, bid, *ptr.clone(), cnt, memo, new_values, changed);
            let new_expr = Expr {
                ty,
                kind: ExprKind::Load(*volatile, Box::new(ptr)),
            };
            hoist_if_needed(func, new_expr, cnt, memo, new_values, changed)
        }
        ExprKind::Store(volatile, ptr, value) => {
            let ptr = rewrite(m, func, bid, *ptr.clone(), cnt, memo, new_values, changed);
            let value = rewrite(m, func, bid, *value.clone(), cnt, memo, new_values, changed);
            let new_expr = Expr {
                ty,
                kind: ExprKind::Store(*volatile, Box::new(ptr), Box::new(value)),
            };
            hoist_if_needed(func, new_expr, cnt, memo, new_values, changed)
        }
        ExprKind::PtrOffset(base, offset) => {
            let base = rewrite(m, func, bid, *base.clone(), cnt, memo, new_values, changed);
            let offset = rewrite(
                m,
                func,
                bid,
                *offset.clone(),
                cnt,
                memo,
                new_values,
                changed,
            );
            let new_expr = Expr {
                ty,
                kind: ExprKind::PtrOffset(Box::new(base), Box::new(offset)),
            };
            hoist_if_needed(func, new_expr, cnt, memo, new_values, changed)
        }
        ExprKind::ElementPtr(elem_ty, base, offset) => {
            let base = rewrite(m, func, bid, *base.clone(), cnt, memo, new_values, changed);
            let offset = rewrite(
                m,
                func,
                bid,
                *offset.clone(),
                cnt,
                memo,
                new_values,
                changed,
            );
            let new_expr = Expr {
                ty,
                kind: ExprKind::ElementPtr(*elem_ty, Box::new(base), Box::new(offset)),
            };
            hoist_if_needed(func, new_expr, cnt, memo, new_values, changed)
        }

        ExprKind::Add(a, b) => {
            let a = rewrite(m, func, bid, *a.clone(), cnt, memo, new_values, changed);
            let b = rewrite(m, func, bid, *b.clone(), cnt, memo, new_values, changed);
            let new_expr = Expr {
                ty,
                kind: ExprKind::Add(Box::new(a), Box::new(b)),
            };
            hoist_if_needed(func, new_expr, cnt, memo, new_values, changed)
        }

        ExprKind::Sub(a, b) => {
            let a = rewrite(m, func, bid, *a.clone(), cnt, memo, new_values, changed);
            let b = rewrite(m, func, bid, *b.clone(), cnt, memo, new_values, changed);
            let new_expr = Expr {
                ty,
                kind: ExprKind::Sub(Box::new(a), Box::new(b)),
            };
            hoist_if_needed(func, new_expr, cnt, memo, new_values, changed)
        }

        ExprKind::Mul(a, b) => {
            let a = rewrite(m, func, bid, *a.clone(), cnt, memo, new_values, changed);
            let b = rewrite(m, func, bid, *b.clone(), cnt, memo, new_values, changed);
            let new_expr = Expr {
                ty,
                kind: ExprKind::Mul(Box::new(a), Box::new(b)),
            };
            hoist_if_needed(func, new_expr, cnt, memo, new_values, changed)
        }

        ExprKind::Div(signed, a, b) => {
            let a = rewrite(m, func, bid, *a.clone(), cnt, memo, new_values, changed);
            let b = rewrite(m, func, bid, *b.clone(), cnt, memo, new_values, changed);
            let new_expr = Expr {
                ty,
                kind: ExprKind::Div(*signed, Box::new(a), Box::new(b)),
            };
            hoist_if_needed(func, new_expr, cnt, memo, new_values, changed)
        }

        ExprKind::Rem(signed, a, b) => {
            let a = rewrite(m, func, bid, *a.clone(), cnt, memo, new_values, changed);
            let b = rewrite(m, func, bid, *b.clone(), cnt, memo, new_values, changed);
            let new_expr = Expr {
                ty,
                kind: ExprKind::Rem(*signed, Box::new(a), Box::new(b)),
            };
            hoist_if_needed(func, new_expr, cnt, memo, new_values, changed)
        }

        ExprKind::Pow(a) => {
            let a = rewrite(m, func, bid, *a.clone(), cnt, memo, new_values, changed);
            let new_expr = Expr {
                ty,
                kind: ExprKind::Pow(Box::new(a)),
            };
            hoist_if_needed(func, new_expr, cnt, memo, new_values, changed)
        }

        ExprKind::FAdd(a, b) => {
            let a = rewrite(m, func, bid, *a.clone(), cnt, memo, new_values, changed);
            let b = rewrite(m, func, bid, *b.clone(), cnt, memo, new_values, changed);
            let new_expr = Expr {
                ty,
                kind: ExprKind::FAdd(Box::new(a), Box::new(b)),
            };
            hoist_if_needed(func, new_expr, cnt, memo, new_values, changed)
        }

        ExprKind::FSub(a, b) => {
            let a = rewrite(m, func, bid, *a.clone(), cnt, memo, new_values, changed);
            let b = rewrite(m, func, bid, *b.clone(), cnt, memo, new_values, changed);
            let new_expr = Expr {
                ty,
                kind: ExprKind::FSub(Box::new(a), Box::new(b)),
            };
            hoist_if_needed(func, new_expr, cnt, memo, new_values, changed)
        }

        ExprKind::FMul(a, b) => {
            let a = rewrite(m, func, bid, *a.clone(), cnt, memo, new_values, changed);
            let b = rewrite(m, func, bid, *b.clone(), cnt, memo, new_values, changed);
            let new_expr = Expr {
                ty,
                kind: ExprKind::FMul(Box::new(a), Box::new(b)),
            };
            hoist_if_needed(func, new_expr, cnt, memo, new_values, changed)
        }

        ExprKind::FDiv(a, b) => {
            let a = rewrite(m, func, bid, *a.clone(), cnt, memo, new_values, changed);
            let b = rewrite(m, func, bid, *b.clone(), cnt, memo, new_values, changed);
            let new_expr = Expr {
                ty,
                kind: ExprKind::FDiv(Box::new(a), Box::new(b)),
            };
            hoist_if_needed(func, new_expr, cnt, memo, new_values, changed)
        }

        ExprKind::FRem(a, b) => {
            let a = rewrite(m, func, bid, *a.clone(), cnt, memo, new_values, changed);
            let b = rewrite(m, func, bid, *b.clone(), cnt, memo, new_values, changed);
            let new_expr = Expr {
                ty,
                kind: ExprKind::FRem(Box::new(a), Box::new(b)),
            };
            hoist_if_needed(func, new_expr, cnt, memo, new_values, changed)
        }

        ExprKind::FPow(a) => {
            let a = rewrite(m, func, bid, *a.clone(), cnt, memo, new_values, changed);
            let new_expr = Expr {
                ty,
                kind: ExprKind::FPow(Box::new(a)),
            };
            hoist_if_needed(func, new_expr, cnt, memo, new_values, changed)
        }

        ExprKind::BitShl(a, b) => {
            let a = rewrite(m, func, bid, *a.clone(), cnt, memo, new_values, changed);
            let b = rewrite(m, func, bid, *b.clone(), cnt, memo, new_values, changed);
            let new_expr = Expr {
                ty,
                kind: ExprKind::BitShl(Box::new(a), Box::new(b)),
            };
            hoist_if_needed(func, new_expr, cnt, memo, new_values, changed)
        }

        ExprKind::BitShr(a, b) => {
            let a = rewrite(m, func, bid, *a.clone(), cnt, memo, new_values, changed);
            let b = rewrite(m, func, bid, *b.clone(), cnt, memo, new_values, changed);
            let new_expr = Expr {
                ty,
                kind: ExprKind::BitShr(Box::new(a), Box::new(b)),
            };
            hoist_if_needed(func, new_expr, cnt, memo, new_values, changed)
        }

        ExprKind::ArithShr(a, b) => {
            let a = rewrite(m, func, bid, *a.clone(), cnt, memo, new_values, changed);
            let b = rewrite(m, func, bid, *b.clone(), cnt, memo, new_values, changed);
            let new_expr = Expr {
                ty,
                kind: ExprKind::ArithShr(Box::new(a), Box::new(b)),
            };
            hoist_if_needed(func, new_expr, cnt, memo, new_values, changed)
        }

        ExprKind::BitAnd(a, b) => {
            let a = rewrite(m, func, bid, *a.clone(), cnt, memo, new_values, changed);
            let b = rewrite(m, func, bid, *b.clone(), cnt, memo, new_values, changed);
            let new_expr = Expr {
                ty,
                kind: ExprKind::BitAnd(Box::new(a), Box::new(b)),
            };
            hoist_if_needed(func, new_expr, cnt, memo, new_values, changed)
        }

        ExprKind::BitOr(a, b) => {
            let a = rewrite(m, func, bid, *a.clone(), cnt, memo, new_values, changed);
            let b = rewrite(m, func, bid, *b.clone(), cnt, memo, new_values, changed);
            let new_expr = Expr {
                ty,
                kind: ExprKind::BitOr(Box::new(a), Box::new(b)),
            };
            hoist_if_needed(func, new_expr, cnt, memo, new_values, changed)
        }

        ExprKind::BitXor(a, b) => {
            let a = rewrite(m, func, bid, *a.clone(), cnt, memo, new_values, changed);
            let b = rewrite(m, func, bid, *b.clone(), cnt, memo, new_values, changed);
            let new_expr = Expr {
                ty,
                kind: ExprKind::BitXor(Box::new(a), Box::new(b)),
            };
            hoist_if_needed(func, new_expr, cnt, memo, new_values, changed)
        }

        ExprKind::BitNeg(a) => {
            let a = rewrite(m, func, bid, *a.clone(), cnt, memo, new_values, changed);
            let new_expr = Expr {
                ty,
                kind: ExprKind::BitNeg(Box::new(a)),
            };
            hoist_if_needed(func, new_expr, cnt, memo, new_values, changed)
        }

        ExprKind::Cmp(kind, a, b) => {
            let a = rewrite(m, func, bid, *a.clone(), cnt, memo, new_values, changed);
            let b = rewrite(m, func, bid, *b.clone(), cnt, memo, new_values, changed);
            let new_expr = Expr {
                ty,
                kind: ExprKind::Cmp(*kind, Box::new(a), Box::new(b)),
            };
            hoist_if_needed(func, new_expr, cnt, memo, new_values, changed)
        }

        ExprKind::FCmp(kind, a, b) => {
            let a = rewrite(m, func, bid, *a.clone(), cnt, memo, new_values, changed);
            let b = rewrite(m, func, bid, *b.clone(), cnt, memo, new_values, changed);
            let new_expr = Expr {
                ty,
                kind: ExprKind::FCmp(*kind, Box::new(a), Box::new(b)),
            };
            hoist_if_needed(func, new_expr, cnt, memo, new_values, changed)
        }
    }
}

fn hoist_if_needed(
    func: &mut FunctionDef,
    expr: Expr,
    cnt: &mut HashMap<Expr, usize>,
    memo: &mut HashMap<Expr, ValueId>,
    new_values: &mut Vec<ValueId>,
    changed: &mut bool,
) -> Expr {
    if let Some(&vid) = memo.get(&expr) {
        return Expr {
            ty: expr.ty,
            kind: ExprKind::Var(vid),
        };
    }

    let uses = cnt.get(&expr).copied().unwrap_or(0);

    if uses <= 1 {
        return expr;
    }

    let vid = func.next_value;
    func.next_value += 1;

    func.values.insert(vid, expr.clone());

    new_values.push(vid);
    memo.insert(expr.clone(), vid);

    *changed = true;

    Expr {
        ty: expr.ty,
        kind: ExprKind::Var(vid),
    }
}
