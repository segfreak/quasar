#![allow(clippy::collapsible_if)]

use crate::*;

pub fn dce(func: &mut FunctionDef) -> bool {
    let mut used = HashSet::<ValueId>::new();

    for (vid, val) in func.values.iter() {
        if val.expr.is_memory() {
            // pre-use for memory operations
            if used.insert(*vid) {
                mark_expr(func, &val.expr.clone(), &mut used);
            }
        }
    }

    for block in func.blocks.values() {
        if let Some(term) = &block.terminator {
            match term {
                Terminator::Ret(v) => {
                    if used.insert(*v) {
                        if let Some(val) = func.values.get(v) {
                            mark_expr(func, &val.expr.clone(), &mut used);
                        }
                    }
                }
                Terminator::Br { params, .. } => {
                    for v in params {
                        if used.insert(*v) {
                            if let Some(val) = func.values.get(v) {
                                mark_expr(func, &val.expr.clone(), &mut used);
                            }
                        }
                    }
                }

                Terminator::BrIf {
                    cond,
                    then_params,
                    else_params,
                    ..
                } => {
                    if used.insert(*cond) {
                        if let Some(val) = func.values.get(cond) {
                            mark_expr(func, &val.expr.clone(), &mut used);
                        }
                    }

                    for v in then_params {
                        if used.insert(*v) {
                            if let Some(val) = func.values.get(v) {
                                mark_expr(func, &val.expr.clone(), &mut used);
                            }
                        }
                    }
                    for v in else_params {
                        if used.insert(*v) {
                            if let Some(val) = func.values.get(v) {
                                mark_expr(func, &val.expr.clone(), &mut used);
                            }
                        }
                    }
                }
            }
        }
    }

    let old_value_count = func.values.len();

    func.values.retain(|vid, _| used.contains(vid));

    for block in func.blocks.values_mut() {
        block.params.retain(|v| used.contains(v));
        block.values.retain(|v| used.contains(v));

        if let Some(Terminator::Ret(v)) = &block.terminator {
            if !used.contains(v) {
                block.terminator = None;
            }
        }
    }

    old_value_count != func.values.len()
}

fn mark_expr(func: &FunctionDef, expr: &Expr, used: &mut std::collections::HashSet<ValueId>) {
    match expr {
        Expr::Var(v) => {
            if used.insert(*v) {
                if let Some(val) = func.values.get(v) {
                    mark_expr(func, &val.expr, used);
                }
            }
        }

        Expr::Const(_) => {}
        Expr::Alloca(_) => {}

        Expr::BitNeg(a) | Expr::Load(_, a) => {
            mark_expr(func, a, used);
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
            mark_expr(func, a, used);
            mark_expr(func, b, used);
        }
    }
}
