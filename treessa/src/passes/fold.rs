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
        Expr::Var(_)
        | Expr::Const(_)
        | Expr::Alloca(_)
        | Expr::Load(_, _)
        | Expr::Store(_, _, _)
        | Expr::PtrOffset(_, _)
        | Expr::ElementPtr(_, _, _) => expr,

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

        Expr::Div(signed, a, b) => {
            let a = fold_expr(*a, changed);
            let b = fold_expr(*b, changed);
            if let (Expr::Const(x), Expr::Const(y)) = (&a, &b) {
                *changed = true;
                if signed {
                    Expr::Const(x / y)
                } else {
                    Expr::Const((*x as u64 / *y as u64) as i64)
                }
            } else {
                Expr::Div(signed, Box::new(a), Box::new(b))
            }
        }

        Expr::BitShl(a, b) => {
            let a = fold_expr(*a, changed);
            let b = fold_expr(*b, changed);
            Expr::BitShl(Box::new(a), Box::new(b))
        }
        Expr::BitShr(a, b) => {
            let a = fold_expr(*a, changed);
            let b = fold_expr(*b, changed);
            Expr::BitShr(Box::new(a), Box::new(b))
        }
        Expr::ArithShr(a, b) => {
            let a = fold_expr(*a, changed);
            let b = fold_expr(*b, changed);
            Expr::ArithShr(Box::new(a), Box::new(b))
        }
        Expr::BitAnd(a, b) => {
            let a = fold_expr(*a, changed);
            let b = fold_expr(*b, changed);
            Expr::BitAnd(Box::new(a), Box::new(b))
        }
        Expr::BitOr(a, b) => {
            let a = fold_expr(*a, changed);
            let b = fold_expr(*b, changed);
            Expr::BitOr(Box::new(a), Box::new(b))
        }
        Expr::BitXor(a, b) => {
            let a = fold_expr(*a, changed);
            let b = fold_expr(*b, changed);
            Expr::BitXor(Box::new(a), Box::new(b))
        }
        Expr::BitNeg(a) => {
            let a = fold_expr(*a, changed);
            Expr::BitNeg(Box::new(a))
        }

        Expr::Cmp(kind, a, b) => {
            let a = fold_expr(*a, changed);
            let b = fold_expr(*b, changed);
            Expr::Cmp(kind, Box::new(a), Box::new(b))
        }

        Expr::FCmp(kind, a, b) => {
            let a = fold_expr(*a, changed);
            let b = fold_expr(*b, changed);
            Expr::FCmp(kind, Box::new(a), Box::new(b))
        }
    }
}
