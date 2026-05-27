use crate::tree::*;

pub fn fold(func: &mut FunctionDef) -> bool {
    let mut changed = false;
    for expr in func.values.values_mut() {
        *expr = fold_expr(expr.clone(), &mut changed);
    }
    changed
}

pub fn fold_expr(expr: Expr, changed: &mut bool) -> Expr {
    let ty = expr.ty;
    let kind = match expr.kind {
        ExprKind::Var(_)
        | ExprKind::Const(_)
        | ExprKind::Alloca(_)
        | ExprKind::NAlloca(_, _)
        | ExprKind::Load(_, _)
        | ExprKind::Store(_, _, _)
        | ExprKind::PtrOffset(_, _)
        | ExprKind::ElementPtr(_, _, _)
        | ExprKind::Cast(_, _) => expr.kind,

        ExprKind::Call(func, params) => {
            let folded: Vec<Expr> = params
                .iter()
                .map(|expr| fold_expr(expr.clone(), changed))
                .collect();
            ExprKind::Call(func, folded)
        }

        ExprKind::Add(a, b) => {
            let a = fold_expr(*a, changed);
            let b = fold_expr(*b, changed);
            if let (ExprKind::Const(x), ExprKind::Const(y)) = (&a.kind, &b.kind) {
                *changed = true;
                ExprKind::Const(x + y)
            } else {
                ExprKind::Add(Box::new(a), Box::new(b))
            }
        }

        ExprKind::Sub(a, b) => {
            let a = fold_expr(*a, changed);
            let b = fold_expr(*b, changed);
            if let (ExprKind::Const(x), ExprKind::Const(y)) = (&a.kind, &b.kind) {
                *changed = true;
                ExprKind::Const(x - y)
            } else {
                ExprKind::Sub(Box::new(a), Box::new(b))
            }
        }

        ExprKind::Mul(a, b) => {
            let a = fold_expr(*a, changed);
            let b = fold_expr(*b, changed);
            if let (ExprKind::Const(x), ExprKind::Const(y)) = (&a.kind, &b.kind) {
                *changed = true;
                ExprKind::Const(x * y)
            } else {
                ExprKind::Mul(Box::new(a), Box::new(b))
            }
        }

        ExprKind::Div(signed, a, b) => {
            let a = fold_expr(*a, changed);
            let b = fold_expr(*b, changed);
            if let (ExprKind::Const(x), ExprKind::Const(y)) = (&a.kind, &b.kind) {
                *changed = true;
                if signed {
                    ExprKind::Const(x / y)
                } else {
                    ExprKind::Const((*x as u64 / *y as u64) as i64)
                }
            } else {
                ExprKind::Div(signed, Box::new(a), Box::new(b))
            }
        }

        ExprKind::Rem(signed, a, b) => {
            let a = fold_expr(*a, changed);
            let b = fold_expr(*b, changed);
            if let (ExprKind::Const(x), ExprKind::Const(y)) = (&a.kind, &b.kind) {
                *changed = true;
                if signed {
                    ExprKind::Const(x % y)
                } else {
                    ExprKind::Const((*x as u64 % *y as u64) as i64)
                }
            } else {
                ExprKind::Div(signed, Box::new(a), Box::new(b))
            }
        }

        ExprKind::FAdd(a, b) => {
            let a = fold_expr(*a, changed);
            let b = fold_expr(*b, changed);
            log::debug!("float-folding is not implemented yet");
            ExprKind::FAdd(Box::new(a), Box::new(b))
        }
        ExprKind::FSub(a, b) => {
            let a = fold_expr(*a, changed);
            let b = fold_expr(*b, changed);
            log::debug!("float-folding is not implemented yet");
            ExprKind::FSub(Box::new(a), Box::new(b))
        }
        ExprKind::FMul(a, b) => {
            let a = fold_expr(*a, changed);
            let b = fold_expr(*b, changed);
            log::debug!("float-folding is not implemented yet");
            ExprKind::FMul(Box::new(a), Box::new(b))
        }
        ExprKind::FDiv(a, b) => {
            let a = fold_expr(*a, changed);
            let b = fold_expr(*b, changed);
            log::debug!("float-folding is not implemented yet");
            ExprKind::FDiv(Box::new(a), Box::new(b))
        }
        ExprKind::FRem(a, b) => {
            let a = fold_expr(*a, changed);
            let b = fold_expr(*b, changed);
            log::debug!("float-folding is not implemented yet");
            ExprKind::FRem(Box::new(a), Box::new(b))
        }

        ExprKind::BitShl(a, b) => {
            let a = fold_expr(*a, changed);
            let b = fold_expr(*b, changed);
            ExprKind::BitShl(Box::new(a), Box::new(b))
        }
        ExprKind::BitShr(a, b) => {
            let a = fold_expr(*a, changed);
            let b = fold_expr(*b, changed);
            ExprKind::BitShr(Box::new(a), Box::new(b))
        }
        ExprKind::ArithShr(a, b) => {
            let a = fold_expr(*a, changed);
            let b = fold_expr(*b, changed);
            ExprKind::ArithShr(Box::new(a), Box::new(b))
        }
        ExprKind::BitAnd(a, b) => {
            let a = fold_expr(*a, changed);
            let b = fold_expr(*b, changed);
            ExprKind::BitAnd(Box::new(a), Box::new(b))
        }
        ExprKind::BitOr(a, b) => {
            let a = fold_expr(*a, changed);
            let b = fold_expr(*b, changed);
            ExprKind::BitOr(Box::new(a), Box::new(b))
        }
        ExprKind::BitXor(a, b) => {
            let a = fold_expr(*a, changed);
            let b = fold_expr(*b, changed);
            ExprKind::BitXor(Box::new(a), Box::new(b))
        }
        ExprKind::BitNeg(a) => {
            let a = fold_expr(*a, changed);
            ExprKind::BitNeg(Box::new(a))
        }

        ExprKind::Cmp(kind, a, b) => {
            let a = fold_expr(*a, changed);
            let b = fold_expr(*b, changed);
            ExprKind::Cmp(kind, Box::new(a), Box::new(b))
        }

        ExprKind::FCmp(kind, a, b) => {
            let a = fold_expr(*a, changed);
            let b = fold_expr(*b, changed);
            ExprKind::FCmp(kind, Box::new(a), Box::new(b))
        }
    };

    Expr { ty, kind }
}
