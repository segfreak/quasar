// normalization,canonicalization
use crate::tree::*;

pub fn normalize(func: &mut FunctionDef) -> bool {
    let mut changed = false;

    for value in func.values.values_mut() {
        value.expr = normalize_expr(value.expr.clone(), &mut changed);
    }

    changed
}

fn normalize_expr(expr: Expr, changed: &mut bool) -> Expr {
    match expr {
        Expr::Var(_) | Expr::Const(_) | Expr::Alloca(_) => expr,

        Expr::Call(func, params) => {
            let normalized: Vec<Expr> = params
                .iter()
                .map(|expr| normalize_expr(expr.clone(), changed))
                .collect();
            Expr::Call(func, normalized)
        }

        Expr::Load(volatile, ptr) => {
            let ptr = normalize_expr(*ptr, changed);
            Expr::Load(volatile, Box::new(ptr))
        }

        Expr::Store(volatile, ptr, value) => {
            let ptr = normalize_expr(*ptr, changed);
            let value = normalize_expr(*value, changed);
            Expr::Store(volatile, Box::new(ptr), Box::new(value))
        }

        Expr::PtrOffset(base, offset) => {
            let base = normalize_expr(*base, changed);
            let offset = normalize_expr(*offset, changed);
            Expr::PtrOffset(Box::new(base), Box::new(offset))
        }

        Expr::ElementPtr(ty, base, offset) => {
            let base = normalize_expr(*base, changed);
            let offset = normalize_expr(*offset, changed);
            Expr::ElementPtr(ty, Box::new(base), Box::new(offset))
        }

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

        Expr::Div(signed, a, b) => {
            let a = normalize_expr(*a, changed);
            let b = normalize_expr(*b, changed);
            Expr::Div(signed, Box::new(a), Box::new(b))
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

        Expr::ArithShr(a, b) => {
            let a = normalize_expr(*a, changed);
            let b = normalize_expr(*b, changed);

            Expr::ArithShr(Box::new(a), Box::new(b))
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

        Expr::Cmp(kind, a, b) => {
            let a = normalize_expr(*a, changed);
            let b = normalize_expr(*b, changed);
            Expr::Cmp(kind, Box::new(a), Box::new(b))
        }

        Expr::FCmp(kind, a, b) => {
            let a = normalize_expr(*a, changed);
            let b = normalize_expr(*b, changed);
            Expr::FCmp(kind, Box::new(a), Box::new(b))
        }
    }
}
