use crate::tree::*;

pub fn canonicalize(func: &mut FunctionDef) -> bool {
    let mut changed = false;
    for expr in func.values.values_mut() {
        *expr = canonicalize_expr(expr.clone(), &mut changed);
    }
    changed
}

pub fn canonicalize_expr(expr: Expr, changed: &mut bool) -> Expr {
    let ty = expr.ty;

    let kind = match expr.kind {
        ExprKind::Var(_)
        | ExprKind::Const(_)
        | ExprKind::FConst(_)
        | ExprKind::Alloca(_)
        | ExprKind::NAlloca(_, _) => expr.kind,

        ExprKind::Undef => expr.kind,

        ExprKind::Load(volatile, ptr) => {
            let a = canonicalize_expr(*ptr, changed);
            ExprKind::Load(volatile, Box::new(a))
        }

        ExprKind::Store(volatile, ptr, value) => {
            let a = canonicalize_expr(*ptr, changed);
            let b = canonicalize_expr(*value, changed);
            ExprKind::Store(volatile, Box::new(a), Box::new(b))
        }

        ExprKind::PtrOffset(base, offset) => {
            let a = canonicalize_expr(*base, changed);
            let b = canonicalize_expr(*offset, changed);
            ExprKind::PtrOffset(Box::new(a), Box::new(b))
        }

        ExprKind::ElementPtr(ty, base, offset) => {
            let a = canonicalize_expr(*base, changed);
            let b = canonicalize_expr(*offset, changed);
            ExprKind::ElementPtr(ty, Box::new(a), Box::new(b))
        }

        ExprKind::Call(func, params) => {
            let folded: Vec<Expr> = params
                .iter()
                .map(|expr| canonicalize_expr(expr.clone(), changed))
                .collect();
            ExprKind::Call(func, folded)
        }

        ExprKind::Add(a, b) => {
            let a = canonicalize_expr(*a, changed);
            let b = canonicalize_expr(*b, changed);
            ExprKind::Add(Box::new(a), Box::new(b))
        }

        ExprKind::Sub(a, b) => {
            let a = canonicalize_expr(*a, changed);
            let b = canonicalize_expr(*b, changed);
            ExprKind::Sub(Box::new(a), Box::new(b))
        }

        ExprKind::Mul(a, b) => {
            let a = canonicalize_expr(*a, changed);
            let b = canonicalize_expr(*b, changed);
            ExprKind::Mul(Box::new(a), Box::new(b))
        }

        ExprKind::Div(signed, a, b) => {
            let a = canonicalize_expr(*a, changed);
            let b = canonicalize_expr(*b, changed);
            ExprKind::Div(signed, Box::new(a), Box::new(b))
        }

        ExprKind::Rem(signed, a, b) => {
            let a = canonicalize_expr(*a, changed);
            let b = canonicalize_expr(*b, changed);
            ExprKind::Rem(signed, Box::new(a), Box::new(b))
        }

        ExprKind::Square(a) => {
            let a = canonicalize_expr(*a, changed);
            *changed = true;
            ExprKind::Mul(Box::new(a.clone()), Box::new(a))
        }

        ExprKind::FAdd(a, b) => {
            let a = canonicalize_expr(*a, changed);
            let b = canonicalize_expr(*b, changed);
            ExprKind::FAdd(Box::new(a), Box::new(b))
        }
        ExprKind::FSub(a, b) => {
            let a = canonicalize_expr(*a, changed);
            let b = canonicalize_expr(*b, changed);
            ExprKind::FSub(Box::new(a), Box::new(b))
        }
        ExprKind::FMul(a, b) => {
            let a = canonicalize_expr(*a, changed);
            let b = canonicalize_expr(*b, changed);
            ExprKind::FMul(Box::new(a), Box::new(b))
        }
        ExprKind::FDiv(a, b) => {
            let a = canonicalize_expr(*a, changed);
            let b = canonicalize_expr(*b, changed);
            ExprKind::FDiv(Box::new(a), Box::new(b))
        }
        ExprKind::FRem(a, b) => {
            let a = canonicalize_expr(*a, changed);
            let b = canonicalize_expr(*b, changed);
            ExprKind::FRem(Box::new(a), Box::new(b))
        }
        ExprKind::FSquare(a) => {
            let a = canonicalize_expr(*a, changed);
            *changed = true;
            ExprKind::FMul(Box::new(a.clone()), Box::new(a))
        }
        ExprKind::BitShl(a, b) => {
            let a = canonicalize_expr(*a, changed);
            let b = canonicalize_expr(*b, changed);
            ExprKind::BitShl(Box::new(a), Box::new(b))
        }
        ExprKind::BitShr(a, b) => {
            let a = canonicalize_expr(*a, changed);
            let b = canonicalize_expr(*b, changed);
            ExprKind::BitShr(Box::new(a), Box::new(b))
        }
        ExprKind::ArithShr(a, b) => {
            let a = canonicalize_expr(*a, changed);
            let b = canonicalize_expr(*b, changed);
            ExprKind::ArithShr(Box::new(a), Box::new(b))
        }
        ExprKind::BitAnd(a, b) => {
            let a = canonicalize_expr(*a, changed);
            let b = canonicalize_expr(*b, changed);
            ExprKind::BitAnd(Box::new(a), Box::new(b))
        }
        ExprKind::BitOr(a, b) => {
            let a = canonicalize_expr(*a, changed);
            let b = canonicalize_expr(*b, changed);
            ExprKind::BitOr(Box::new(a), Box::new(b))
        }
        ExprKind::BitXor(a, b) => {
            let a = canonicalize_expr(*a, changed);
            let b = canonicalize_expr(*b, changed);
            ExprKind::BitXor(Box::new(a), Box::new(b))
        }
        ExprKind::BitNeg(a) => {
            let a = canonicalize_expr(*a, changed);
            ExprKind::BitNeg(Box::new(a))
        }

        ExprKind::Cast(kind, a) => {
            let a = canonicalize_expr(*a, changed);
            ExprKind::Cast(kind, Box::new(a))
        }

        ExprKind::Cmp(kind, a, b) => {
            let a = canonicalize_expr(*a, changed);
            let b = canonicalize_expr(*b, changed);
            ExprKind::Cmp(kind, Box::new(a), Box::new(b))
        }

        ExprKind::FCmp(kind, a, b) => {
            let a = canonicalize_expr(*a, changed);
            let b = canonicalize_expr(*b, changed);
            ExprKind::FCmp(kind, Box::new(a), Box::new(b))
        }
    };

    Expr { ty, kind }
}
