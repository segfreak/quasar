use crate::{tree::*, types::Type};

impl FunctionDef {
    pub fn fmt_expr(expr: &Expr) -> String {
        let ty = expr.ty;
        match &expr.kind {
            ExprKind::Var(v) => format!("%{}", v),
            ExprKind::Const(c) => c.to_string(),
            ExprKind::FConst(bits) => f64::from_bits(*bits).to_string(),
            ExprKind::Add(a, b) => {
                format!("({}.add {} {})", ty, Self::fmt_expr(a), Self::fmt_expr(b))
            }
            ExprKind::Sub(a, b) => {
                format!("({}.sub {} {})", ty, Self::fmt_expr(a), Self::fmt_expr(b))
            }
            ExprKind::Mul(a, b) => {
                format!("({}.mul {} {})", ty, Self::fmt_expr(a), Self::fmt_expr(b))
            }
            ExprKind::Square(a) => {
                format!("({}.square {})", ty, Self::fmt_expr(a))
            }
            ExprKind::Div(signed, a, b) => {
                format!(
                    "({}.{}div {} {})",
                    ty,
                    if *signed { "" } else { "u" },
                    Self::fmt_expr(a),
                    Self::fmt_expr(b)
                )
            }
            ExprKind::Rem(signed, a, b) => {
                format!(
                    "({}.{}rem {} {})",
                    ty,
                    if *signed { "" } else { "u" },
                    Self::fmt_expr(a),
                    Self::fmt_expr(b)
                )
            }
            ExprKind::FAdd(a, b) => {
                format!("({}.add {} {})", ty, Self::fmt_expr(a), Self::fmt_expr(b))
            }
            ExprKind::FSub(a, b) => {
                format!("({}.sub {} {})", ty, Self::fmt_expr(a), Self::fmt_expr(b))
            }
            ExprKind::FMul(a, b) => {
                format!("({}.mul {} {})", ty, Self::fmt_expr(a), Self::fmt_expr(b))
            }
            ExprKind::FDiv(a, b) => {
                format!("({}.div {} {})", ty, Self::fmt_expr(a), Self::fmt_expr(b))
            }
            ExprKind::FRem(a, b) => {
                format!("({}.rem {} {})", ty, Self::fmt_expr(a), Self::fmt_expr(b))
            }
            ExprKind::FSquare(a) => {
                format!("({}.square {})", ty, Self::fmt_expr(a))
            }
            ExprKind::BitShl(a, b) => {
                format!("({}.shl {} {})", ty, Self::fmt_expr(a), Self::fmt_expr(b))
            }
            ExprKind::BitShr(a, b) => {
                format!("({}.shr {} {})", ty, Self::fmt_expr(a), Self::fmt_expr(b))
            }
            ExprKind::ArithShr(a, b) => {
                format!("({}.ashr {} {})", ty, Self::fmt_expr(a), Self::fmt_expr(b))
            }
            ExprKind::BitAnd(a, b) => {
                format!("({}.band {} {})", ty, Self::fmt_expr(a), Self::fmt_expr(b))
            }
            ExprKind::BitOr(a, b) => {
                format!("({}.bor {} {})", ty, Self::fmt_expr(a), Self::fmt_expr(b))
            }
            ExprKind::BitXor(a, b) => {
                format!("({}.bxor {} {})", ty, Self::fmt_expr(a), Self::fmt_expr(b))
            }
            ExprKind::BitNeg(a) => format!("({}.bneg {})", ty, Self::fmt_expr(a)),
            ExprKind::Cmp(kind, a, b) => {
                format!(
                    "(icmp.{} {} {})",
                    kind,
                    Self::fmt_expr(a),
                    Self::fmt_expr(b)
                )
            }
            ExprKind::FCmp(kind, a, b) => {
                format!(
                    "(fcmp.{} {} {})",
                    kind,
                    Self::fmt_expr(a),
                    Self::fmt_expr(b)
                )
            }
            ExprKind::Cast(kind, a) => {
                format!("({}.{} {})", ty, kind, Self::fmt_expr(a))
            }
            ExprKind::Alloca(ty) => format!("({}.alloca)", ty),
            ExprKind::NAlloca(ty, cnt) => format!("({}.alloca {})", ty, cnt),

            ExprKind::Load(volatile, ptr) => {
                format!(
                    "({}.{}load {})",
                    ty,
                    if *volatile { "v" } else { "" },
                    Self::fmt_expr(ptr)
                )
            }
            ExprKind::Store(volatile, ptr, value) => {
                format!(
                    "({}store {} {})",
                    if *volatile { "v" } else { "" },
                    Self::fmt_expr(ptr),
                    Self::fmt_expr(value),
                )
            }
            ExprKind::PtrOffset(base, offset) => {
                format!(
                    "(ptroffset {} {})",
                    Self::fmt_expr(base),
                    Self::fmt_expr(offset),
                )
            }
            ExprKind::ElementPtr(ty, base, offset) => {
                format!(
                    "(elementptr {} {} {})",
                    ty,
                    Self::fmt_expr(base),
                    Self::fmt_expr(offset),
                )
            }
            ExprKind::Call(func, params) => {
                let params: Vec<String> = params.iter().map(Self::fmt_expr).collect();
                format!(
                    "(call {} {})",
                    func,
                    if params.is_empty() {
                        "()".to_string()
                    } else {
                        format!("({})", params.join(" "))
                    }
                )
            }
        }
    }

    fn fmt_term(term: &Terminator) -> String {
        let mut out = String::new();

        match term {
            Terminator::Ret(v) => {
                out.push_str(&format!("    (ret {} {})\n", v.ty, Self::fmt_expr(v)));
            }
            Terminator::RetVoid => {
                out.push_str(&format!("    (ret {})\n", Type::Void));
            }
            Terminator::Br { bb, params } => {
                let params: Vec<String> = params.iter().map(Self::fmt_expr).collect();
                out.push_str(&format!(
                    "    (br B{} {})\n",
                    bb,
                    if params.is_empty() {
                        "()".to_string()
                    } else {
                        format!("({})", params.join(" "))
                    }
                ));
            }
            Terminator::BrIf {
                cond,
                then_bb,
                then_params,
                else_bb,
                else_params,
            } => {
                let cond = Self::fmt_expr(cond);
                let then_params: Vec<String> = then_params.iter().map(Self::fmt_expr).collect();
                let else_params: Vec<String> = else_params.iter().map(Self::fmt_expr).collect();

                let format_params = |p: &[String]| {
                    if p.is_empty() {
                        "()".to_string()
                    } else {
                        format!("({})", p.join(" "))
                    }
                };

                out.push_str(&format!(
                    "    (br-if {} B{} {} B{} {})\n",
                    cond,
                    then_bb,
                    format_params(&then_params),
                    else_bb,
                    format_params(&else_params)
                ));
            }
        }

        out
    }

    pub fn dump(&self) -> String {
        let mut out = String::new();

        let mut block_ids: Vec<BlockId> = self.blocks.keys().copied().collect();
        block_ids.sort();

        for bid in block_ids {
            let block = &self.blocks[&bid];

            let params_str = if block.params.is_empty() {
                "()".to_string()
            } else {
                let params: Vec<String> = block
                    .params
                    .iter()
                    .map(|v| {
                        let ty = &self.values[v].ty;
                        format!("(%{} {})", v, ty)
                    })
                    .collect();
                format!("({})", params.join(" "))
            };

            out.push_str(&format!(
                "(block B{}\n  (params {})\n  (body\n",
                bid, params_str
            ));

            for v in &block.values {
                let val = &self.values[v];
                if val.ty.is_void() {
                    out.push_str(&format!("    {}\n", Self::fmt_expr(val)));
                } else {
                    out.push_str(&format!("    (set %{} {})\n", v, Self::fmt_expr(val)));
                }
            }

            if let Some(term) = &block.terminator {
                out.push_str(&Self::fmt_term(term));
            }

            out.push_str("  )\n)\n\n");
        }

        out
    }
}
