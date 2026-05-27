#![allow(clippy::collapsible_if)]

use std::collections::HashSet;

use crate::tree::*;

pub fn dce(func: &mut FunctionDef) -> bool {
    let mut used = HashSet::<ValueId>::new();

    for (vid, val) in func.values.iter() {
        if val.kind.is_memory() || val.kind.is_call() {
            // pre-use for call or memory operations
            if used.insert(*vid) {
                mark_expr(func, &val.clone(), &mut used);
            }
        }
    }

    for entry_param in func.get_entry_param_exprs() {
        mark_expr(func, &entry_param.clone(), &mut used);
    }

    for block in func.blocks.values() {
        if let Some(term) = &block.terminator {
            match term {
                Terminator::RetVoid => {}

                Terminator::Ret(v) => {
                    mark_expr(func, &v.clone(), &mut used);
                }
                Terminator::Br { params, .. } => {
                    for v in params {
                        mark_expr(func, &v.clone(), &mut used);
                    }
                }

                Terminator::BrIf {
                    cond,
                    then_params,
                    else_params,
                    ..
                } => {
                    mark_expr(func, &cond.clone(), &mut used);

                    for v in then_params {
                        mark_expr(func, &v.clone(), &mut used);
                    }
                    for v in else_params {
                        mark_expr(func, &v.clone(), &mut used);
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
    }

    old_value_count != func.values.len()
}

fn mark_expr(func: &FunctionDef, expr: &Expr, used: &mut std::collections::HashSet<ValueId>) {
    match &expr.kind {
        ExprKind::Var(v) => {
            if used.insert(*v) {
                if let Some(val) = func.values.get(v) {
                    mark_expr(func, val, used);
                }
            }
        }

        ExprKind::Const(_) | ExprKind::FConst(_) => {}
        ExprKind::Alloca(_) => {}
        ExprKind::NAlloca(_, _) => {}

        ExprKind::Call(_, params) => {
            for expr in params {
                mark_expr(func, expr, used);
            }
        }

        ExprKind::Cast(_, a) | ExprKind::BitNeg(a) | ExprKind::Load(_, a) => {
            mark_expr(func, a, used);
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
            mark_expr(func, a, used);
            mark_expr(func, b, used);
        }
    }
}
