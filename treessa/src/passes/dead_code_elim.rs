#![allow(clippy::collapsible_if)]

use crate::*;

pub fn dead_code_elim(func: &mut FunctionDef) -> bool {
    let mut used = HashSet::<ValueId>::new();

    for block in func.blocks.values() {
        if let Some(Terminator::Ret(v)) = &block.terminator {
            if used.insert(*v) {
                if let Some(val) = func.values.get(v) {
                    mark_expr(func, &val.expr.clone(), &mut used);
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

        Expr::Add(a, b) | Expr::Sub(a, b) | Expr::Mul(a, b) => {
            mark_expr(func, a, used);
            mark_expr(func, b, used);
        }
    }
}
