// strength reduction, decompose
use crate::{tree::*, types::Type};

pub fn strength_reduction(func: &mut FunctionDef) -> bool {
    let mut changed = false;
    for expr in func.values.values_mut() {
        *expr = reduce_expr(expr.clone(), &mut changed);
    }
    changed
}

pub fn reduce_expr(expr: Expr, changed: &mut bool) -> Expr {
    let ty = expr.ty;
    let cost = expr.get_cost();

    let kind = match expr.kind {
        ExprKind::Call(func, params) => {
            let reducted: Vec<Expr> = params
                .iter()
                .map(|expr| reduce_expr(expr.clone(), changed))
                .collect();
            ExprKind::Call(func, reducted)
        }

        ExprKind::Add(a, b) => {
            let a = reduce_expr(*a, changed);
            let b = reduce_expr(*b, changed);
            ExprKind::Add(Box::new(a), Box::new(b))
        }

        ExprKind::Sub(a, b) => {
            let a = reduce_expr(*a, changed);
            let b = reduce_expr(*b, changed);
            ExprKind::Sub(Box::new(a), Box::new(b))
        }

        ExprKind::Mul(a, b) => {
            let a = reduce_expr(*a, changed);
            let b = reduce_expr(*b, changed);

            match (&a.kind, &b.kind) {
                // mul => shifts
                (ExprKind::Const(y), _) | (_, ExprKind::Const(y)) if is_power_of_two(*y) => {
                    *changed = true;
                    let shifts = y.trailing_zeros() as i64;
                    let (x, other_y) = if matches!(a.kind, ExprKind::Const(_)) {
                        (&b, a)
                    } else {
                        (&a, b)
                    };
                    return Expr {
                        ty,
                        kind: ExprKind::BitShl(
                            Box::new(x.clone()),
                            Box::new(Expr {
                                ty: other_y.ty,
                                kind: ExprKind::Const(shifts),
                            }),
                        ),
                    };
                }

                // mul => shifts (decompose)
                (ExprKind::Const(y), _) | (_, ExprKind::Const(y))
                    if !is_power_of_two(*y) && *y > 2 =>
                {
                    let x = if matches!(a.kind, ExprKind::Const(_)) {
                        &b
                    } else {
                        &a
                    };
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
                    ExprKind::Mul(Box::new(a), Box::new(b))
                }

                _ => ExprKind::Mul(Box::new(a), Box::new(b)),
            }
        }

        ExprKind::Div(signed, a, b) if !signed => {
            let a = reduce_expr(*a, changed);
            let b = reduce_expr(*b, changed);

            match &b.kind {
                // div => shifts
                ExprKind::Const(y) if is_power_of_two(*y) => {
                    *changed = true;
                    let shifts = y.trailing_zeros() as i64;
                    return Expr {
                        ty,
                        kind: ExprKind::BitShr(
                            Box::new(a),
                            Box::new(Expr {
                                ty: b.ty,
                                kind: ExprKind::Const(shifts),
                            }),
                        ),
                    };
                }
                _ => ExprKind::Div(signed, Box::new(a), Box::new(b)),
            }
        }

        ExprKind::Rem(signed, a, b) if !signed => {
            let a = reduce_expr(*a, changed);
            let b = reduce_expr(*b, changed);

            match &b.kind {
                // rem => shifts
                ExprKind::Const(y) if is_power_of_two(*y) => {
                    *changed = true;
                    let mask = y - 1;
                    return Expr {
                        ty,
                        kind: ExprKind::BitAnd(
                            Box::new(a),
                            Box::new(Expr {
                                ty: b.ty,
                                kind: ExprKind::Const(mask),
                            }),
                        ),
                    };
                }
                _ => ExprKind::Rem(signed, Box::new(a), Box::new(b)),
            }
        }

        other => other,
    };

    Expr { ty, kind }
}

fn try_decompose_mul(x: Expr, c: i64) -> Option<Expr> {
    if let ExprKind::Mul(_left, right) = x.kind.clone() {
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

        let mut result = make_lshift(x.clone(), right.ty, shifts[0]);

        for &shift in &shifts[1..] {
            let next_shift_expr = make_lshift(x.clone(), right.ty, shift);
            result = Expr {
                ty: x.ty,
                kind: ExprKind::Add(Box::new(result), Box::new(next_shift_expr)),
            };
        }

        Some(result)
    } else {
        None
    }
}

fn is_power_of_two(v: i64) -> bool {
    v > 0 && (v & (v - 1)) == 0
}

fn make_lshift(x: Expr, shift_ty: Type, shift: i64) -> Expr {
    if shift == 0 {
        x
    } else {
        Expr {
            ty: x.ty,
            kind: ExprKind::BitShl(
                Box::new(x),
                Box::new(Expr {
                    ty: shift_ty,
                    kind: ExprKind::Const(shift),
                }),
            ),
        }
    }
}
