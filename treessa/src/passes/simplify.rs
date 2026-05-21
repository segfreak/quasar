// algebraic simplify, algebraic reassociation
use crate::*;

pub fn simplify(func: &mut FunctionDef) -> bool {
    let mut changed = false;
    for val in func.values.values_mut() {
        let old = std::mem::replace(&mut val.expr, Expr::Const(0));
        val.expr = simplify_expr(old.clone(), &mut changed);
    }
    changed
}

pub fn simplify_expr(expr: Expr, changed: &mut bool) -> Expr {
    match expr {
        Expr::Add(a, b) => {
            let a = simplify_expr(*a, changed);
            let b = simplify_expr(*b, changed);

            match (&a, &b) {
                (Expr::Const(0), x) | (x, Expr::Const(0)) => {
                    *changed = true;
                    x.clone()
                }
                // algebraic reassociation
                // (x + const y) + const z  =>  x + const (y + z)
                (Expr::Add(box x, box Expr::Const(y)), Expr::Const(z)) => {
                    *changed = true;
                    Expr::Add(Box::new(x.clone()), Box::new(Expr::Const(y + z)))
                }
                // algebraic reassociation
                // (x - const y) + const z  =>  x - const (y - z)
                (Expr::Sub(box x, box Expr::Const(y)), Expr::Const(z)) => {
                    *changed = true;
                    Expr::Sub(Box::new(x.clone()), Box::new(Expr::Const(y - z)))
                }
                _ => Expr::Add(Box::new(a), Box::new(b)),
            }
        }

        Expr::Sub(a, b) => {
            let a = simplify_expr(*a, changed);
            let b = simplify_expr(*b, changed);

            if a == b {
                Expr::Const(0)
            } else {
                match (&a, &b) {
                    (x, Expr::Const(0)) => {
                        *changed = true;
                        x.clone()
                    }
                    // algebraic reassociation
                    // (x + const y) - const z  =>  x + const (y - z)
                    (Expr::Add(box x, box Expr::Const(y)), Expr::Const(z)) => {
                        *changed = true;
                        Expr::Add(Box::new(x.clone()), Box::new(Expr::Const(y - z)))
                    }
                    // algebraic reassociation
                    // (x - const y) - const z  =>  x - const (y + z)
                    (Expr::Sub(box x, box Expr::Const(y)), Expr::Const(z)) => {
                        *changed = true;
                        Expr::Sub(Box::new(x.clone()), Box::new(Expr::Const(y + z)))
                    }
                    _ => Expr::Sub(Box::new(a), Box::new(b)),
                }
            }
        }

        Expr::Mul(a, b) => {
            let a = simplify_expr(*a, changed);
            let b = simplify_expr(*b, changed);

            match (&a, &b) {
                // 0 * x => 0
                // x * 0 => 0
                (Expr::Const(0), _) | (_, Expr::Const(0)) => {
                    *changed = true;
                    Expr::Const(0)
                }
                // 1 * x => x
                // x * 1 => x
                (Expr::Const(1), x) | (x, Expr::Const(1)) => {
                    *changed = true;
                    x.clone()
                }
                // algebraic reassociation
                // (x * const y) * const z => x * const (y * z)
                (Expr::Mul(box x, box Expr::Const(y)), Expr::Const(z)) => {
                    *changed = true;
                    Expr::Mul(Box::new(x.clone()), Box::new(Expr::Const(y * z)))
                }
                _ => Expr::Mul(Box::new(a), Box::new(b)),
            }
        }

        Expr::Div(a, b) => {
            let a = simplify_expr(*a, changed);
            let b = simplify_expr(*b, changed);

            match (&a, &b) {
                (_, Expr::Const(0)) => {
                    log::trace!("division by zero");
                    Expr::Div(Box::new(a), Box::new(b))
                }
                // x / 1 => x
                (x, Expr::Const(1)) => {
                    *changed = true;
                    x.clone()
                }
                // (x * const y) / const z => x * const (y / z)
                (Expr::Mul(box x, box Expr::Const(y)), Expr::Const(z)) if y % z == 0 => {
                    *changed = true;
                    Expr::Mul(Box::new(x.clone()), Box::new(Expr::Const(y / z)))
                }
                // (x * const y) / const z => x * const (y / z)
                (Expr::Mul(box x, box Expr::Const(y)), Expr::Const(z)) if z % y == 0 => {
                    *changed = true;
                    Expr::Div(Box::new(x.clone()), Box::new(Expr::Const(z / y)))
                }
                (Expr::Add(box x, box Expr::Const(y)), Expr::Const(z)) if y % z == 0 => {
                    *changed = true;
                    Expr::Add(
                        Box::new(Expr::Div(Box::new(x.clone()), Box::new(Expr::Const(*z)))),
                        Box::new(Expr::Const(y / z)),
                    )
                }
                (Expr::Sub(box x, box Expr::Const(y)), Expr::Const(z)) if y % z == 0 => {
                    *changed = true;
                    Expr::Sub(
                        Box::new(Expr::Div(Box::new(x.clone()), Box::new(Expr::Const(*z)))),
                        Box::new(Expr::Const(y / z)),
                    )
                }
                _ => Expr::Div(Box::new(a), Box::new(b)),
            }
        }

        other => other,
    }
}
