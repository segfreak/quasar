// strength reduction, decompose
use crate::*;

pub fn strength_reduction(func: &mut FunctionDef) -> bool {
    let mut changed = false;
    for val in func.values.values_mut() {
        let old = std::mem::replace(&mut val.expr, Expr::Const(0));
        val.expr = reduce_expr(old.clone(), &mut changed);
    }
    changed
}

pub fn reduce_expr(expr: Expr, changed: &mut bool) -> Expr {
    let cost = expr.get_cost();

    match expr {
        Expr::Add(a, b) => {
            let a = reduce_expr(*a, changed);
            let b = reduce_expr(*b, changed);
            Expr::Add(Box::new(a), Box::new(b))
        }

        Expr::Sub(a, b) => {
            let a = reduce_expr(*a, changed);
            let b = reduce_expr(*b, changed);
            Expr::Sub(Box::new(a), Box::new(b))
        }

        Expr::Mul(a, b) => {
            let a = reduce_expr(*a, changed);
            let b = reduce_expr(*b, changed);

            match (&a, &b) {
                // x * 2 => x + x
                (x, Expr::Const(2)) | (Expr::Const(2), x) => {
                    *changed = true;
                    Expr::Add(Box::new(x.clone()), Box::new(x.clone()))
                }

                // mul => shifts
                (x, Expr::Const(y)) | (Expr::Const(y), x) if is_power_of_two(*y) => {
                    *changed = true;
                    let shifts = y.trailing_zeros() as i64;
                    Expr::BitShl(Box::new(x.clone()), Box::new(Expr::Const(shifts)))
                }

                // mul => shifts (decompose)
                (x, Expr::Const(y)) if !is_power_of_two(*y) && *y > 2 => {
                    if let Some(new) = try_decompose_mul(x.clone(), *y) {
                        let new_cost = new.get_cost();
                        if new_cost < cost {
                            *changed = true;
                            return new;
                        }
                        log::debug!(
                            "decompose for constant {} is not profitable (cost {} >= {})",
                            y,
                            new_cost,
                            cost
                        );
                    }
                    Expr::Mul(Box::new(a), Box::new(b))
                }

                _ => Expr::Mul(Box::new(a), Box::new(b)),
            }
        }

        Expr::Div(a, b) => {
            let a = reduce_expr(*a, changed);
            let b = reduce_expr(*b, changed);

            match (&a, &b) {
                // div => shifts
                (x, Expr::Const(y)) if is_power_of_two(*y) => {
                    *changed = true;
                    let shifts = y.trailing_zeros() as i64;
                    Expr::BitShr(Box::new(x.clone()), Box::new(Expr::Const(shifts)))
                }
                _ => Expr::Div(Box::new(a), Box::new(b)),
            }
        }

        other => other,
    }
}

fn try_decompose_mul(x: Expr, c: i64) -> Option<Expr> {
    if c <= 2 || (c & (c - 1)) == 0 {
        return None;
    }

    let mut current_val = c;
    let mut shifts = Vec::new();
    let mut bit_position = 0;

    while current_val > 0 {
        if (current_val & 1) == 1 {
            shifts.push(bit_position);
        }
        current_val >>= 1;
        bit_position += 1;
    }

    let mut result = make_lshift(x.clone(), shifts[0]);

    for &shift in &shifts[1..] {
        let next_shift_expr = make_lshift(x.clone(), shift);
        result = Expr::Add(Box::new(result), Box::new(next_shift_expr));
    }

    Some(result)
}

fn is_power_of_two(v: i64) -> bool {
    v > 0 && (v & (v - 1)) == 0
}

fn make_lshift(x: Expr, shift: i64) -> Expr {
    if shift == 0 {
        x
    } else {
        Expr::BitShl(Box::new(x), Box::new(Expr::Const(shift)))
    }
}
