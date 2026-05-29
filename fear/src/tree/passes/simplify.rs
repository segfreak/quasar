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
                // (x + y) + z  =>  x + (y + z)
                (ExprKind::Add(box x, box y), _) => {
                    *changed = true;
                    ExprKind::Add(
                        Box::new(x.clone()),
                        Box::new(Expr {
                            ty: expr.ty,
                            kind: ExprKind::Add(Box::new(y.clone()), Box::new(b)),
                        }),
                    )
                }
                // (x - y) + z  =>  x - (y - z)
                (ExprKind::Sub(box x, box y), _) => {
                    *changed = true;
                    ExprKind::Sub(
                        Box::new(x.clone()),
                        Box::new(Expr {
                            ty: expr.ty,
                            kind: ExprKind::Sub(Box::new(y.clone()), Box::new(b)),
                        }),
                    )
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
                    // (x + y) - z  =>  x + (y - z)
                    (ExprKind::Add(box x, box y), _) => {
                        *changed = true;
                        ExprKind::Add(
                            Box::new(x.clone()),
                            Box::new(Expr {
                                ty: expr.ty,
                                kind: ExprKind::Sub(Box::new(y.clone()), Box::new(b)),
                            }),
                        )
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

            if a == b {
                *changed = true;
                return Expr {
                    ty: expr.ty,
                    kind: ExprKind::Const(0),
                };
            }

            match (&a.kind, &b.kind) {
                // x & 0 => 0
                (ExprKind::Const(0), _) | (_, ExprKind::Const(0)) => {
                    *changed = true;
                    ExprKind::Const(0)
                }

                _ => ExprKind::BitAnd(Box::new(a), Box::new(b)),
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

        // ExprKind::Pow(a) => {
        //     let a = simplify_expr(*a, changed);
        //     ExprKind::Mul(Box::new(a.clone()), Box::new(a))
        // }
        other => other,
    };

    Expr { ty: expr.ty, kind }
}
