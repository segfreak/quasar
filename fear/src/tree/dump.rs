use crate::{tree::*, types::Type};

impl FunctionDef {
    pub fn fmt_expr(&self, expr: &Expr) -> String {
        match &expr.kind {
            ExprKind::Var(v) => format!("%{}", v),
            ExprKind::Const(c) => c.to_string(),
            ExprKind::Add(a, b) => format!("add({}, {})", self.fmt_expr(a), self.fmt_expr(b)),
            ExprKind::Sub(a, b) => format!("sub({}, {})", self.fmt_expr(a), self.fmt_expr(b)),
            ExprKind::Mul(a, b) => format!("mul({}, {})", self.fmt_expr(a), self.fmt_expr(b)),
            ExprKind::Div(_, a, b) => {
                format!("div({}, {})", self.fmt_expr(a), self.fmt_expr(b))
            }
            ExprKind::Rem(_, a, b) => {
                format!("rem({}, {})", self.fmt_expr(a), self.fmt_expr(b))
            }
            ExprKind::BitShl(a, b) => format!("shl({}, {})", self.fmt_expr(a), self.fmt_expr(b)),
            ExprKind::BitShr(a, b) => format!("shr({}, {})", self.fmt_expr(a), self.fmt_expr(b)),
            ExprKind::ArithShr(a, b) => format!("ashr({}, {})", self.fmt_expr(a), self.fmt_expr(b)),
            ExprKind::BitAnd(a, b) => format!("band({}, {})", self.fmt_expr(a), self.fmt_expr(b)),
            ExprKind::BitOr(a, b) => format!("bor({}, {})", self.fmt_expr(a), self.fmt_expr(b)),
            ExprKind::BitXor(a, b) => format!("bxor({}, {})", self.fmt_expr(a), self.fmt_expr(b)),
            ExprKind::BitNeg(a) => format!("bneg({})", self.fmt_expr(a)),
            ExprKind::Cmp(kind, a, b) => {
                format!("icmp {}({}, {})", kind, self.fmt_expr(a), self.fmt_expr(b))
            }
            ExprKind::FCmp(kind, a, b) => {
                format!("fcmp {}( {}, {})", kind, self.fmt_expr(a), self.fmt_expr(b))
            }
            ExprKind::Alloca(ty) => format!("alloca({})", ty),
            ExprKind::Load(volatile, ptr) => {
                format!(
                    "{}load({})",
                    if *volatile { "v" } else { "" },
                    self.fmt_expr(ptr)
                )
            }
            ExprKind::Store(volatile, ptr, value) => {
                format!(
                    "{}store({}, {})",
                    if *volatile { "v" } else { "" },
                    self.fmt_expr(ptr),
                    self.fmt_expr(value),
                )
            }
            ExprKind::PtrOffset(base, offset) => {
                format!(
                    "ptroffset({}, {})",
                    self.fmt_expr(base),
                    self.fmt_expr(offset),
                )
            }
            ExprKind::ElementPtr(ty, base, offset) => {
                format!(
                    "elementptr({}, {}, {})",
                    ty,
                    self.fmt_expr(base),
                    self.fmt_expr(offset),
                )
            }
            ExprKind::Call(func, params) => {
                let params: Vec<String> = params.iter().map(|expr| self.fmt_expr(expr)).collect();
                format!("m.call({}, {})", func, params.join(", "))
            }
        }
    }

    fn fmt_term(&self, term: &Terminator) -> String {
        let mut out = String::new();

        match term {
            Terminator::Ret(v) => {
                out.push_str(&format!("  ret {} {}\n", v.ty, self.fmt_expr(v)));
            }
            Terminator::RetVoid => {
                out.push_str(&format!("  ret {}\n", Type::Void));
            }
            Terminator::Br { bb, params } => {
                let params: Vec<String> = params.iter().map(|expr| self.fmt_expr(expr)).collect();
                out.push_str(&format!("  br B{}({})\n", bb, params.join(", ")));
            }
            Terminator::BrIf {
                cond,
                then_bb,
                then_params,
                else_bb,
                else_params,
            } => {
                let cond = self.fmt_expr(cond);
                let then_params: Vec<String> =
                    then_params.iter().map(|expr| self.fmt_expr(expr)).collect();
                let else_params: Vec<String> =
                    else_params.iter().map(|expr| self.fmt_expr(expr)).collect();

                out.push_str(&format!(
                    "  brif %{} B{}({}), B{}({})\n",
                    cond,
                    then_bb,
                    then_params.join(", "),
                    else_bb,
                    else_params.join(", ")
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

            if block.params.is_empty() {
                out.push_str(&format!("B{}:\n", bid));
            } else {
                let params: Vec<String> = block
                    .params
                    .iter()
                    .map(|v| {
                        let ty = &self.values[v].ty;
                        format!("%{}: {}", v, ty)
                    })
                    .collect();
                out.push_str(&format!("B{}({}):\n", bid, params.join(", ")));
            }

            for v in &block.values {
                let val = &self.values[v];
                out.push_str(&format!("  %{} = {}\n", v, self.fmt_expr(&val)));
            }

            if let Some(term) = &block.terminator {
                out.push_str(&self.fmt_term(term))
            }

            out.push('\n');
        }

        out
    }
}
