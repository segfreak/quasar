// algebraic simplify, algebraic reassociation
use crate::{tree::*, types::CastKind};

pub fn simplify(func: &mut FunctionDef) -> bool {
    let mut changed = false;
    for val in func.values.values_mut() {
        *val = simplify_expr(val.clone(), &mut changed);
    }
    changed
}

pub fn simplify_expr(expr: Expr, changed: &mut bool) -> Expr {
    let kind = match expr.kind {
        ExprKind::Const(_)
        | ExprKind::FConst(_)
        | ExprKind::Var(_)
        | ExprKind::Alloca(_)
        | ExprKind::NAlloca(_, _)
        | ExprKind::Undef => expr.kind,

        ExprKind::Neg(a) => {
            let a = simplify_expr(*a, changed);
            ExprKind::Neg(Box::new(a))
        }

        ExprKind::FNeg(a) => {
            let a = simplify_expr(*a, changed);
            ExprKind::Neg(Box::new(a))
        }

        ExprKind::Select(c, t, e) => {
            let c = simplify_expr(*c, changed);
            let t = simplify_expr(*t, changed);
            let e = simplify_expr(*e, changed);

            match c.kind {
                ExprKind::Const(1) => t.kind,
                ExprKind::Const(0) => e.kind,
                _ => ExprKind::Select(Box::new(c), Box::new(t), Box::new(e)),
            }
        }

        ExprKind::Cmp(pred, l, r) => {
            let l = simplify_expr(*l, changed);
            let r = simplify_expr(*r, changed);
            ExprKind::Cmp(pred, Box::new(l), Box::new(r))
        }

        ExprKind::FCmp(pred, l, r) => {
            let l = simplify_expr(*l, changed);
            let r = simplify_expr(*r, changed);
            ExprKind::FCmp(pred, Box::new(l), Box::new(r))
        }

        ExprKind::Call(func, params) => {
            let simplified: Vec<Expr> = params
                .iter()
                .map(|expr| simplify_expr(expr.clone(), changed))
                .collect();
            ExprKind::Call(func, simplified)
        }

        // (x * y) + (x2 * z) if x == x2  =>  x * (y + z)
        ExprKind::Add(
            box Expr {
                kind: ExprKind::Mul(box x, box y),
                ..
            },
            box Expr {
                kind: ExprKind::Mul(box x2, box z),
                ..
            },
        ) if x == x2 => {
            *changed = true;
            ExprKind::Mul(
                Box::new(x),
                Box::new(Expr {
                    ty: expr.ty,
                    kind: ExprKind::Add(Box::new(y), Box::new(z)),
                }),
            )
        }

        ExprKind::Add(a, b) => {
            let a = simplify_expr(*a, changed);
            let b = simplify_expr(*b, changed);

            match (&a.kind, &b.kind) {
                (_, ExprKind::BitNeg(y)) => {
                    *changed = true;
                    ExprKind::Sub(Box::new(a), Box::new(*y.clone()))
                }
                (ExprKind::Const(0), _) | (_, ExprKind::Const(0)) => {
                    *changed = true;
                    let result = if matches!(a.kind, ExprKind::Const(0)) {
                        b
                    } else {
                        a
                    };
                    return result;
                }
                // add(mul(x, y), x) => x * (y + 1)
                (ExprKind::Mul(box x1, box y), _) if x1 == &b => {
                    *changed = true;
                    ExprKind::Mul(
                        Box::new(x1.clone()),
                        Box::new(Expr {
                            ty: expr.ty,
                            kind: ExprKind::Add(
                                Box::new(y.clone()),
                                Box::new(Expr {
                                    ty: expr.ty,
                                    kind: ExprKind::Const(1),
                                }),
                            ),
                        }),
                    )
                }
                // add(x, mul(x, y)) => x * (y + 1)
                (_, ExprKind::Mul(box x1, box y)) if x1 == &a => {
                    *changed = true;
                    ExprKind::Mul(
                        Box::new(x1.clone()),
                        Box::new(Expr {
                            ty: expr.ty,
                            kind: ExprKind::Add(
                                Box::new(y.clone()),
                                Box::new(Expr {
                                    ty: expr.ty,
                                    kind: ExprKind::Const(1),
                                }),
                            ),
                        }),
                    )
                }
                // (x - y) + y => x
                (ExprKind::Sub(box x, box y1), b) if y1.kind == *b => {
                    *changed = true;
                    return x.clone();
                }
                _ => ExprKind::Add(Box::new(a), Box::new(b)),
            }
        }

        ExprKind::Sub(a, b) => {
            let a = simplify_expr(*a, changed);
            let b = simplify_expr(*b, changed);

            if a == b {
                ExprKind::Const(0)
            } else {
                match (&a.kind, &b.kind) {
                    (_, ExprKind::Const(0)) => {
                        *changed = true;
                        return a;
                    }
                    // x^2 - y^2 = (x - y) * (x + y)
                    (ExprKind::Square(x), ExprKind::Square(y)) => {
                        *changed = true;
                        ExprKind::Mul(
                            Box::new(Expr {
                                ty: expr.ty,
                                kind: ExprKind::Sub(Box::new(*x.clone()), Box::new(*y.clone())),
                            }),
                            Box::new(Expr {
                                ty: expr.ty,
                                kind: ExprKind::Add(Box::new(*x.clone()), Box::new(*y.clone())),
                            }),
                        )
                    }
                    // (x + y) - y => x
                    (ExprKind::Add(box x, box y1), b) if y1.kind == *b => {
                        *changed = true;
                        return x.clone();
                    }
                    // (x - y) - z  =>  x - (y + z)
                    (ExprKind::Sub(box x, box y), _) => {
                        *changed = true;
                        ExprKind::Sub(
                            Box::new(x.clone()),
                            Box::new(Expr {
                                ty: expr.ty,
                                kind: ExprKind::Add(Box::new(y.clone()), Box::new(b)),
                            }),
                        )
                    }
                    _ => ExprKind::Sub(Box::new(a), Box::new(b)),
                }
            }
        }

        ExprKind::Mul(a, b) => {
            let a = simplify_expr(*a, changed);
            let b = simplify_expr(*b, changed);

            match (&a.kind, &b.kind) {
                (_, ExprKind::Const(-1)) => {
                    *changed = true;
                    ExprKind::BitNeg(Box::new(a))
                }

                // 0 * x => 0
                // x * 0 => 0
                (ExprKind::Const(0), _) | (_, ExprKind::Const(0)) => {
                    *changed = true;
                    ExprKind::Const(0)
                }
                // 1 * x => x
                // x * 1 => x
                (ExprKind::Const(1), _) | (_, ExprKind::Const(1)) => {
                    *changed = true;
                    let result = if matches!(a.kind, ExprKind::Const(1)) {
                        b
                    } else {
                        a
                    };
                    return result;
                }
                // (x * const y) * const z => x * const (y * z)
                (
                    ExprKind::Mul(
                        box x,
                        box Expr {
                            kind: ExprKind::Const(y),
                            ..
                        },
                    ),
                    ExprKind::Const(z),
                ) => {
                    *changed = true;
                    ExprKind::Mul(
                        Box::new(x.clone()),
                        Box::new(Expr {
                            ty: expr.ty,
                            kind: ExprKind::Const(y * z),
                        }),
                    )
                }
                (
                    ExprKind::Div(
                        _,
                        box x,
                        box Expr {
                            kind: ExprKind::Const(y),
                            ..
                        },
                    ),
                    ExprKind::Const(z),
                ) if y == z => {
                    *changed = true;
                    return x.clone();
                }
                _ => ExprKind::Mul(Box::new(a), Box::new(b)),
            }
        }

        ExprKind::Square(a) => {
            let a = simplify_expr(*a, changed);
            ExprKind::Square(Box::new(a))
        }

        ExprKind::Div(signed, a, b) => {
            let a = simplify_expr(*a, changed);
            let b = simplify_expr(*b, changed);

            match (&a.kind, &b.kind) {
                (_, ExprKind::Const(0)) => {
                    log::trace!("division by zero");
                    ExprKind::Div(signed, Box::new(a), Box::new(b))
                }
                // x / 1 => x
                (_, ExprKind::Const(1)) => {
                    *changed = true;
                    return a;
                }
                // (x * const y) / const z => x * const (y / z)
                (
                    ExprKind::Mul(
                        box x,
                        box Expr {
                            kind: ExprKind::Const(y),
                            ..
                        },
                    ),
                    ExprKind::Const(z),
                ) if y % z == 0 => {
                    *changed = true;
                    ExprKind::Mul(
                        Box::new(x.clone()),
                        Box::new(Expr {
                            ty: expr.ty,
                            kind: ExprKind::Const(y / z),
                        }),
                    )
                }
                // (x * const y) / const z => x / const (z / y)
                (
                    ExprKind::Mul(
                        box x,
                        box Expr {
                            kind: ExprKind::Const(y),
                            ..
                        },
                    ),
                    ExprKind::Const(z),
                ) if z % y == 0 => {
                    *changed = true;
                    ExprKind::Div(
                        signed,
                        Box::new(x.clone()),
                        Box::new(Expr {
                            ty: expr.ty,
                            kind: ExprKind::Const(z / y),
                        }),
                    )
                }
                (
                    ExprKind::Add(
                        box x,
                        box Expr {
                            kind: ExprKind::Const(y),
                            ..
                        },
                    ),
                    ExprKind::Const(z),
                ) if y % z == 0 => {
                    *changed = true;
                    ExprKind::Add(
                        Box::new(Expr {
                            ty: expr.ty,
                            kind: ExprKind::Div(
                                signed,
                                Box::new(x.clone()),
                                Box::new(Expr {
                                    ty: expr.ty,
                                    kind: ExprKind::Const(*z),
                                }),
                            ),
                        }),
                        Box::new(Expr {
                            ty: expr.ty,
                            kind: ExprKind::Const(y / z),
                        }),
                    )
                }
                (
                    ExprKind::Sub(
                        box x,
                        box Expr {
                            kind: ExprKind::Const(y),
                            ..
                        },
                    ),
                    ExprKind::Const(z),
                ) if y % z == 0 => {
                    *changed = true;
                    ExprKind::Sub(
                        Box::new(Expr {
                            ty: expr.ty,
                            kind: ExprKind::Div(
                                signed,
                                Box::new(x.clone()),
                                Box::new(Expr {
                                    ty: expr.ty,
                                    kind: ExprKind::Const(*z),
                                }),
                            ),
                        }),
                        Box::new(Expr {
                            ty: expr.ty,
                            kind: ExprKind::Const(y / z),
                        }),
                    )
                }
                _ => ExprKind::Div(signed, Box::new(a), Box::new(b)),
            }
        }

        ExprKind::Rem(signed, a, b) => {
            let a = simplify_expr(*a, changed);
            let b = simplify_expr(*b, changed);
            ExprKind::Rem(signed, Box::new(a), Box::new(b))
        }

        ExprKind::Cast(kind, a) => {
            let a = simplify_expr(*a, changed);
            if expr.ty == a.ty
                && matches!(
                    kind,
                    CastKind::Zext | CastKind::Sext | CastKind::Trunc | CastKind::Bitcast
                )
            {
                a.kind
            } else {
                ExprKind::Cast(kind, Box::new(a))
            }
        }

        ExprKind::BitAnd(a, b) => {
            let a = simplify_expr(*a, changed);
            let b = simplify_expr(*b, changed);

            match (&a.kind, &b.kind) {
                // x & 0 => 0
                (ExprKind::Const(0), _) | (_, ExprKind::Const(0)) => {
                    *changed = true;
                    ExprKind::Const(0)
                }
                _ => ExprKind::BitAnd(Box::new(a), Box::new(b)),
            }
        }

        ExprKind::BitOr(a, b) => {
            let a = simplify_expr(*a, changed);
            let b = simplify_expr(*b, changed);

            match (&a.kind, &b.kind) {
                // x | 0 => x
                (ExprKind::Const(0), a) | (a, ExprKind::Const(0)) => {
                    *changed = true;
                    a.clone()
                }
                // x | x => x
                (a, b) | (b, a) if a == b => {
                    *changed = true;
                    a.clone()
                }
                _ => ExprKind::BitOr(Box::new(a), Box::new(b)),
            }
        }

        ExprKind::BitXor(a, b) => {
            let a = simplify_expr(*a, changed);
            let b = simplify_expr(*b, changed);

            if a == b {
                *changed = true;
                return Expr {
                    ty: expr.ty,
                    kind: ExprKind::Const(0),
                };
            }

            match (&a.kind, &b.kind) {
                // (x ^ y) ^ z if x == z => y
                (ExprKind::BitXor(box x, box y), _) if x == &b => {
                    *changed = true;
                    return y.clone();
                }
                // (x ^ y) ^ z if y == z => x
                (ExprKind::BitXor(box x, box y), _) if y == &b => {
                    *changed = true;
                    return x.clone();
                }
                _ => ExprKind::BitXor(Box::new(a), Box::new(b)),
            }
        }

        ExprKind::BitNeg(a) => {
            let a = simplify_expr(*a, changed);
            ExprKind::BitNeg(Box::new(a))
        }

        ExprKind::BitShl(a, b) => {
            let a = simplify_expr(*a, changed);
            let b = simplify_expr(*b, changed);
            ExprKind::BitShl(Box::new(a), Box::new(b))
        }

        ExprKind::BitShr(a, b) => {
            let a = simplify_expr(*a, changed);
            let b = simplify_expr(*b, changed);
            ExprKind::BitShr(Box::new(a), Box::new(b))
        }

        ExprKind::ArithShr(a, b) => {
            let a = simplify_expr(*a, changed);
            let b = simplify_expr(*b, changed);
            ExprKind::ArithShr(Box::new(a), Box::new(b))
        }

        ExprKind::Load(volatile, ptr) => {
            let ptr = simplify_expr(*ptr, changed);
            ExprKind::Load(volatile, Box::new(ptr))
        }

        ExprKind::Store(volatile, ptr, value) => {
            let ptr = simplify_expr(*ptr, changed);
            let value = simplify_expr(*value, changed);
            ExprKind::Store(volatile, Box::new(ptr), Box::new(value))
        }

        ExprKind::PtrOffset(ptr, offset) => {
            let ptr = simplify_expr(*ptr, changed);
            let offset = simplify_expr(*offset, changed);
            ExprKind::PtrOffset(Box::new(ptr), Box::new(offset))
        }

        ExprKind::ElementPtr(ty, ptr, offset) => {
            let ptr = simplify_expr(*ptr, changed);
            let offset = simplify_expr(*offset, changed);
            ExprKind::ElementPtr(ty, Box::new(ptr), Box::new(offset))
        }

        ExprKind::FAdd(a, b) => {
            let a = simplify_expr(*a, changed);
            let b = simplify_expr(*b, changed);
            ExprKind::FAdd(Box::new(a), Box::new(b))
        }

        ExprKind::FSub(a, b) => {
            let a = simplify_expr(*a, changed);
            let b = simplify_expr(*b, changed);
            ExprKind::FSub(Box::new(a), Box::new(b))
        }

        ExprKind::FMul(a, b) => {
            let a = simplify_expr(*a, changed);
            let b = simplify_expr(*b, changed);
            ExprKind::FMul(Box::new(a), Box::new(b))
        }

        ExprKind::FDiv(a, b) => {
            let a = simplify_expr(*a, changed);
            let b = simplify_expr(*b, changed);
            ExprKind::FDiv(Box::new(a), Box::new(b))
        }

        ExprKind::FRem(a, b) => {
            let a = simplify_expr(*a, changed);
            let b = simplify_expr(*b, changed);
            ExprKind::FRem(Box::new(a), Box::new(b))
        }

        ExprKind::FSquare(a) => {
            let a = simplify_expr(*a, changed);
            ExprKind::FSquare(Box::new(a))
        }
    };

    Expr { ty: expr.ty, kind }
}
