// normalization,canonicalization
use crate::*;

pub fn normalize(func: &mut FunctionDef) -> bool {
    let mut changed = false;

    for value in func.values.values_mut() {
        value.expr = normalize_expr(value.expr.clone(), &mut changed);
    }

    changed
}

fn normalize_expr(expr: Expr, changed: &mut bool) -> Expr {
    match expr {
        Expr::Add(a, b) => {
            let a = normalize_expr(*a, changed);
            let b = normalize_expr(*b, changed);

            // add(x, x) -> mul(x, 2)
            if a == b {
                *changed = true;

                return Expr::Mul(Box::new(a), Box::new(Expr::Const(2)));
            }

            // canonical constant ordering
            // add(const, x) -> add(x, const)
            match (&a, &b) {
                (Expr::Const(_), _) => {
                    *changed = true;

                    Expr::Add(Box::new(b), Box::new(a))
                }

                _ => Expr::Add(Box::new(a), Box::new(b)),
            }
        }

        Expr::Sub(a, b) => {
            let a = normalize_expr(*a, changed);
            let b = normalize_expr(*b, changed);

            Expr::Sub(Box::new(a), Box::new(b))
        }

        Expr::Mul(a, b) => {
            let a = normalize_expr(*a, changed);
            let b = normalize_expr(*b, changed);

            match (&a, &b) {
                (Expr::Const(_), _) => {
                    *changed = true;

                    Expr::Mul(Box::new(b), Box::new(a))
                }

                _ => Expr::Mul(Box::new(a), Box::new(b)),
            }
        }

        Expr::Div(a, b) => {
            let a = normalize_expr(*a, changed);
            let b = normalize_expr(*b, changed);

            Expr::Div(Box::new(a), Box::new(b))
        }

        Expr::BitShl(a, b) => {
            let a = normalize_expr(*a, changed);
            let b = normalize_expr(*b, changed);

            match &b {
                // shl(x, C) -> mul(x, 1<<C)
                Expr::Const(shift) if *shift >= 0 && *shift < 63 => {
                    *changed = true;

                    Expr::Mul(Box::new(a), Box::new(Expr::Const(1i64 << shift)))
                }

                _ => Expr::BitShl(Box::new(a), Box::new(b)),
            }
        }

        Expr::BitShr(a, b) => {
            let a = normalize_expr(*a, changed);
            let b = normalize_expr(*b, changed);

            Expr::BitShr(Box::new(a), Box::new(b))
        }

        Expr::BitNeg(a) => {
            let a = normalize_expr(*a, changed);
            *changed = true;
            Expr::Mul(Box::new(a), Box::new(Expr::Const(-1)))
        }

        Expr::BitAnd(a, b) => {
            let a = normalize_expr(*a, changed);
            let b = normalize_expr(*b, changed);
            Expr::BitAnd(Box::new(a), Box::new(b))
        }

        Expr::BitOr(a, b) => {
            let a = normalize_expr(*a, changed);
            let b = normalize_expr(*b, changed);
            Expr::BitOr(Box::new(a), Box::new(b))
        }

        Expr::BitXor(a, b) => {
            let a = normalize_expr(*a, changed);
            let b = normalize_expr(*b, changed);
            Expr::BitXor(Box::new(a), Box::new(b))
        }

        other => other,
    }
}
