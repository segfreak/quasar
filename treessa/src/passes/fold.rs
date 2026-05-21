use crate::{Expr, FunctionDef};

pub fn fold(func: &mut FunctionDef) -> bool {
    let mut changed = false;
    for val in func.values.values_mut() {
        let old = std::mem::replace(&mut val.expr, Expr::Const(0));
        val.expr = fold_expr(old.clone(), &mut changed);
    }
    changed
}

pub fn fold_expr(expr: Expr, changed: &mut bool) -> Expr {
    match expr {
        Expr::Add(a, b) => {
            let a = fold_expr(*a, changed);
            let b = fold_expr(*b, changed);
            if let (Expr::Const(x), Expr::Const(y)) = (&a, &b) {
                *changed = true;
                Expr::Const(x + y)
            } else {
                Expr::Add(Box::new(a), Box::new(b))
            }
        }

        Expr::Sub(a, b) => {
            let a = fold_expr(*a, changed);
            let b = fold_expr(*b, changed);
            if let (Expr::Const(x), Expr::Const(y)) = (&a, &b) {
                *changed = true;
                Expr::Const(x - y)
            } else {
                Expr::Sub(Box::new(a), Box::new(b))
            }
        }

        Expr::Mul(a, b) => {
            let a = fold_expr(*a, changed);
            let b = fold_expr(*b, changed);
            if let (Expr::Const(x), Expr::Const(y)) = (&a, &b) {
                *changed = true;
                Expr::Const(x * y)
            } else {
                Expr::Mul(Box::new(a), Box::new(b))
            }
        }

        Expr::Div(a, b) => {
            let a = fold_expr(*a, changed);
            let b = fold_expr(*b, changed);
            if let (Expr::Const(x), Expr::Const(y)) = (&a, &b) {
                *changed = true;
                Expr::Const(x / y)
            } else {
                Expr::Div(Box::new(a), Box::new(b))
            }
        }

        other => other,
    }
}
