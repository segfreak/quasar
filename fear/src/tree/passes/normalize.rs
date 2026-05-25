// normalization,canonicalization
use crate::tree::*;

pub fn normalize(func: &mut FunctionDef) -> bool {
    let mut changed = false;

    for value in func.values.values_mut() {
        *value = normalize_expr(value.clone(), &mut changed);
    }

    changed
}

fn normalize_expr(expr: Expr, changed: &mut bool) -> Expr {
    let kind = match expr.kind {
        ExprKind::Var(_) | ExprKind::Const(_) | ExprKind::Alloca(_) => expr.kind,

        ExprKind::Call(func, params) => {
            let normalized: Vec<Expr> = params
                .iter()
                .map(|expr| normalize_expr(expr.clone(), changed))
                .collect();
            ExprKind::Call(func, normalized)
        }

        ExprKind::Load(volatile, ptr) => {
            let ptr = normalize_expr(*ptr, changed);
            ExprKind::Load(volatile, Box::new(ptr))
        }

        ExprKind::Store(volatile, ptr, value) => {
            let ptr = normalize_expr(*ptr, changed);
            let value = normalize_expr(*value, changed);
            ExprKind::Store(volatile, Box::new(ptr), Box::new(value))
        }

        ExprKind::PtrOffset(base, offset) => {
            let base = normalize_expr(*base, changed);
            let offset = normalize_expr(*offset, changed);
            ExprKind::PtrOffset(Box::new(base), Box::new(offset))
        }

        ExprKind::ElementPtr(ty, base, offset) => {
            let base = normalize_expr(*base, changed);
            let offset = normalize_expr(*offset, changed);
            ExprKind::ElementPtr(ty, Box::new(base), Box::new(offset))
        }

        ExprKind::Add(a, b) => {
            let a = normalize_expr(*a, changed);
            let b = normalize_expr(*b, changed);

            match (&a.kind, &b.kind) {
                // canonical constant ordering
                // add(const, x) -> add(x, const)
                (ExprKind::Const(_), _) => {
                    *changed = true;

                    ExprKind::Add(Box::new(b), Box::new(a))
                }

                // add(x, x) -> mul(x, 2)
                (x, y) if x == y => ExprKind::Mul(
                    Box::new(a),
                    Box::new(Expr {
                        ty: b.ty,
                        kind: ExprKind::Const(2),
                    }),
                ),

                _ => ExprKind::Add(Box::new(a), Box::new(b)),
            }
        }

        ExprKind::Sub(a, b) => {
            let a = normalize_expr(*a, changed);
            let b = normalize_expr(*b, changed);

            ExprKind::Sub(Box::new(a), Box::new(b))
        }

        ExprKind::Mul(a, b) => {
            let a = normalize_expr(*a, changed);
            let b = normalize_expr(*b, changed);

            match (&a.kind, &b.kind) {
                (ExprKind::Const(_), _) => {
                    *changed = true;

                    ExprKind::Mul(Box::new(b), Box::new(a))
                }

                _ => ExprKind::Mul(Box::new(a), Box::new(b)),
            }
        }

        ExprKind::Div(signed, a, b) => {
            let a = normalize_expr(*a, changed);
            let b = normalize_expr(*b, changed);
            ExprKind::Div(signed, Box::new(a), Box::new(b))
        }

        ExprKind::Rem(signed, a, b) => {
            let a = normalize_expr(*a, changed);
            let b = normalize_expr(*b, changed);
            ExprKind::Rem(signed, Box::new(a), Box::new(b))
        }

        ExprKind::BitShl(a, b) => {
            let a = normalize_expr(*a, changed);
            let b = normalize_expr(*b, changed);

            match &b.kind {
                // shl(x, C) -> mul(x, 1<<C)
                ExprKind::Const(shift) if *shift >= 0 && *shift < 63 => {
                    *changed = true;

                    ExprKind::Mul(
                        Box::new(a),
                        Box::new(Expr {
                            ty: b.ty,
                            kind: ExprKind::Const(1i64 << shift),
                        }),
                    )
                }

                _ => ExprKind::BitShl(Box::new(a), Box::new(b)),
            }
        }

        ExprKind::BitShr(a, b) => {
            let a = normalize_expr(*a, changed);
            let b = normalize_expr(*b, changed);

            ExprKind::BitShr(Box::new(a), Box::new(b))
        }

        ExprKind::ArithShr(a, b) => {
            let a = normalize_expr(*a, changed);
            let b = normalize_expr(*b, changed);

            ExprKind::ArithShr(Box::new(a), Box::new(b))
        }

        ExprKind::BitNeg(a) => {
            let a = normalize_expr(*a, changed);
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
            let a = normalize_expr(*a, changed);
            let b = normalize_expr(*b, changed);
            ExprKind::BitAnd(Box::new(a), Box::new(b))
        }

        ExprKind::BitOr(a, b) => {
            let a = normalize_expr(*a, changed);
            let b = normalize_expr(*b, changed);
            ExprKind::BitOr(Box::new(a), Box::new(b))
        }

        ExprKind::BitXor(a, b) => {
            let a = normalize_expr(*a, changed);
            let b = normalize_expr(*b, changed);
            ExprKind::BitXor(Box::new(a), Box::new(b))
        }

        ExprKind::Cmp(kind, a, b) => {
            let a = normalize_expr(*a, changed);
            let b = normalize_expr(*b, changed);
            ExprKind::Cmp(kind, Box::new(a), Box::new(b))
        }

        ExprKind::FCmp(kind, a, b) => {
            let a = normalize_expr(*a, changed);
            let b = normalize_expr(*b, changed);
            ExprKind::FCmp(kind, Box::new(a), Box::new(b))
        }
    };

    Expr { ty: expr.ty, kind }
}
