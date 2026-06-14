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
        ExprKind::Var(_)
        | ExprKind::Const(_)
        | ExprKind::FConst(_)
        | ExprKind::Undef
        | ExprKind::Alloca(_)
        | ExprKind::NAlloca(_, _) => expr.kind,

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
                (ExprKind::Const(c), _) | (_, ExprKind::Const(c))
                    if *c > 1 && !is_power_of_two(*c) =>
                {
                    let x = if matches!(a.kind, ExprKind::Const(_)) {
                        b.clone()
                    } else {
                        a.clone()
                    };

                    #[allow(clippy::collapsible_if)]
                    if let Some(new) = try_decompose_mul(x, *c) {
                        if new.get_cost() < cost {
                            *changed = true;
                            return new;
                        }
                    }

                    ExprKind::Mul(Box::new(a), Box::new(b))
                }

                _ => ExprKind::Mul(Box::new(a), Box::new(b)),
            }
        }

        ExprKind::Div(signed, a, b) if signed => {
            let a = reduce_expr(*a, changed);
            let b = reduce_expr(*b, changed);

            match &b.kind {
                ExprKind::Const(y) if *y > 0 && is_power_of_two(*y) => {
                    *changed = true;

                    let k = y.trailing_zeros() as i64;

                    let bitwidth = a.ty.get_bitwidth() as i64;
                    let sign_shift = bitwidth - 1;

                    // (x >> sign_shift)
                    let sign = Expr {
                        ty: a.ty,
                        kind: ExprKind::ArithShr(
                            Box::new(a.clone()),
                            Box::new(Expr {
                                ty: b.ty,
                                kind: ExprKind::Const(sign_shift),
                            }),
                        ),
                    };

                    // mask = (2^k - 1)
                    let mask_val: i64 = if k >= 63 {
                        i64::MAX >> 1 // safe
                    } else {
                        (1i64 << k) - 1
                    };

                    let mask = Expr {
                        ty: b.ty,
                        kind: ExprKind::Const(mask_val),
                    };

                    let correction = Expr {
                        ty: a.ty,
                        kind: ExprKind::BitAnd(Box::new(sign), Box::new(mask)),
                    };

                    let shifted = Expr {
                        ty: a.ty,
                        kind: ExprKind::ArithShr(
                            Box::new(a),
                            Box::new(Expr {
                                ty: b.ty,
                                kind: ExprKind::Const(k),
                            }),
                        ),
                    };

                    ExprKind::Add(Box::new(shifted), Box::new(correction))
                }

                _ => ExprKind::Div(true, Box::new(a), Box::new(b)),
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

        ExprKind::Select(c, t, e) => {
            let c = reduce_expr(*c, changed);
            let t = reduce_expr(*t, changed);
            let e = reduce_expr(*e, changed);
            ExprKind::Select(Box::new(c), Box::new(t), Box::new(e))
        }

        ExprKind::Cast(kind, a) => {
            let a = reduce_expr(*a, changed);
            ExprKind::Cast(kind, Box::new(a))
        }

        ExprKind::Load(volatile, ptr) => {
            let ptr = reduce_expr(*ptr, changed);
            ExprKind::Load(volatile, Box::new(ptr))
        }

        ExprKind::Store(volatile, ptr, value) => {
            let ptr = reduce_expr(*ptr, changed);
            let value = reduce_expr(*value, changed);
            ExprKind::Store(volatile, Box::new(ptr), Box::new(value))
        }

        ExprKind::PtrOffset(base, offset) => {
            let base = reduce_expr(*base, changed);
            let offset = reduce_expr(*offset, changed);
            ExprKind::PtrOffset(Box::new(base), Box::new(offset))
        }

        ExprKind::ElementPtr(ty, base, offset) => {
            let base = reduce_expr(*base, changed);
            let offset = reduce_expr(*offset, changed);
            ExprKind::ElementPtr(ty, Box::new(base), Box::new(offset))
        }

        ExprKind::Square(a) => {
            let a = reduce_expr(*a, changed);
            ExprKind::Square(Box::new(a))
        }

        ExprKind::FSquare(a) => {
            let a = reduce_expr(*a, changed);
            ExprKind::FSquare(Box::new(a))
        }

        ExprKind::Div(signed, a, b) => {
            let a = reduce_expr(*a, changed);
            let b = reduce_expr(*b, changed);
            ExprKind::Div(signed, Box::new(a), Box::new(b))
        }

        ExprKind::Rem(signed, a, b) => {
            let a = reduce_expr(*a, changed);
            let b = reduce_expr(*b, changed);
            ExprKind::Rem(signed, Box::new(a), Box::new(b))
        }

        ExprKind::FAdd(a, b) => {
            let a = reduce_expr(*a, changed);
            let b = reduce_expr(*b, changed);
            match (&a.kind, &b.kind) {
                (ExprKind::Const(_), _) => {
                    *changed = true;
                    ExprKind::FAdd(Box::new(b), Box::new(a))
                }
                _ => ExprKind::FAdd(Box::new(a), Box::new(b)),
            }
        }
        ExprKind::FSub(a, b) => {
            let a = reduce_expr(*a, changed);
            let b = reduce_expr(*b, changed);
            ExprKind::FSub(Box::new(a), Box::new(b))
        }
        ExprKind::FMul(a, b) => {
            let a = reduce_expr(*a, changed);
            let b = reduce_expr(*b, changed);
            match (&a.kind, &b.kind) {
                (ExprKind::Const(_), _) => {
                    *changed = true;
                    ExprKind::FMul(Box::new(b), Box::new(a))
                }
                _ => ExprKind::FMul(Box::new(a), Box::new(b)),
            }
        }
        ExprKind::FDiv(a, b) => {
            let a = reduce_expr(*a, changed);
            let b = reduce_expr(*b, changed);
            ExprKind::FDiv(Box::new(a), Box::new(b))
        }
        ExprKind::FRem(a, b) => {
            let a = reduce_expr(*a, changed);
            let b = reduce_expr(*b, changed);
            ExprKind::FRem(Box::new(a), Box::new(b))
        }

        ExprKind::BitShl(a, b) => {
            let a = reduce_expr(*a, changed);
            let b = reduce_expr(*b, changed);
            ExprKind::BitShl(Box::new(a), Box::new(b))
        }

        ExprKind::BitShr(a, b) => {
            let a = reduce_expr(*a, changed);
            let b = reduce_expr(*b, changed);
            ExprKind::BitShr(Box::new(a), Box::new(b))
        }

        ExprKind::ArithShr(a, b) => {
            let a = reduce_expr(*a, changed);
            let b = reduce_expr(*b, changed);
            ExprKind::ArithShr(Box::new(a), Box::new(b))
        }

        ExprKind::BitNeg(a) => {
            let a = reduce_expr(*a, changed);
            *changed = true;
            ExprKind::Mul(
                Box::new(a.clone()),
                Box::new(Expr {
                    ty: a.ty,
                    kind: ExprKind::Const(-1),
                }),
            )
        }

        ExprKind::BitAnd(a, b) => {
            let a = reduce_expr(*a, changed);
            let b = reduce_expr(*b, changed);
            ExprKind::BitAnd(Box::new(a), Box::new(b))
        }

        ExprKind::BitOr(a, b) => {
            let a = reduce_expr(*a, changed);
            let b = reduce_expr(*b, changed);
            ExprKind::BitOr(Box::new(a), Box::new(b))
        }

        ExprKind::BitXor(a, b) => {
            let a = reduce_expr(*a, changed);
            let b = reduce_expr(*b, changed);
            ExprKind::BitXor(Box::new(a), Box::new(b))
        }

        ExprKind::Cmp(kind, a, b) => {
            let a = reduce_expr(*a, changed);
            let b = reduce_expr(*b, changed);
            ExprKind::Cmp(kind, Box::new(a), Box::new(b))
        }

        ExprKind::FCmp(kind, a, b) => {
            let a = reduce_expr(*a, changed);
            let b = reduce_expr(*b, changed);
            ExprKind::FCmp(kind, Box::new(a), Box::new(b))
        }
    };

    Expr { ty, kind }
}

fn try_decompose_mul(x: Expr, c: i64) -> Option<Expr> {
    if c <= 1 || is_power_of_two(c) {
        return None;
    }

    let mut shifts = Vec::new();
    let mut v = c;
    let mut bit = 0;

    while v != 0 {
        if (v & 1) != 0 {
            shifts.push(bit);
        }

        v >>= 1;
        bit += 1;
    }

    if shifts.len() <= 1 {
        return None;
    }

    let mut result = make_lshift(x.clone(), x.ty, shifts[0]);

    for shift in &shifts[1..] {
        result = Expr {
            ty: x.ty,
            kind: ExprKind::Add(
                Box::new(result),
                Box::new(make_lshift(x.clone(), x.ty, *shift)),
            ),
        };
    }

    Some(result)
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
