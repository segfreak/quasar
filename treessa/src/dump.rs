use crate::*;

impl FunctionDef {
    fn fmt_expr(&self, expr: &Expr) -> String {
        match expr {
            Expr::Var(v) => format!("%{}", v),
            Expr::Const(c) => c.to_string(),
            Expr::Add(a, b) => format!("add({}, {})", self.fmt_expr(a), self.fmt_expr(b)),
            Expr::Sub(a, b) => format!("sub({}, {})", self.fmt_expr(a), self.fmt_expr(b)),
            Expr::Mul(a, b) => format!("mul({}, {})", self.fmt_expr(a), self.fmt_expr(b)),
            Expr::Div(a, b) => format!("div({}, {})", self.fmt_expr(a), self.fmt_expr(b)),
            Expr::BitShl(a, b) => format!("shl({}, {})", self.fmt_expr(a), self.fmt_expr(b)),
            Expr::BitShr(a, b) => format!("shr({}, {})", self.fmt_expr(a), self.fmt_expr(b)),
            Expr::ArithShr(a, b) => format!("ashr({}, {})", self.fmt_expr(a), self.fmt_expr(b)),
            Expr::BitAnd(a, b) => format!("band({}, {})", self.fmt_expr(a), self.fmt_expr(b)),
            Expr::BitOr(a, b) => format!("bor({}, {})", self.fmt_expr(a), self.fmt_expr(b)),
            Expr::BitXor(a, b) => format!("bxor({}, {})", self.fmt_expr(a), self.fmt_expr(b)),
            Expr::BitNeg(a) => format!("bneg({})", self.fmt_expr(a)),
        }
    }

    fn fmt_term(&self, term: &Terminator) -> String {
        let mut out = String::new();

        match term {
            Terminator::Ret(v) => {
                let val = &self.values[v];
                out.push_str(&format!("  ret {} %{}\n", val.ty, v));
            }
            Terminator::Br { bb, params } => {
                out.push_str(&format!("  br B{}({:?})\n", bb, params));
            }
            Terminator::BrIf {
                cond,
                then_bb,
                then_params,
                else_bb,
                else_params,
            } => {
                out.push_str(&format!(
                    "  brif %{} B{}({:?}), B{}({:?})\n",
                    cond, then_bb, then_params, else_bb, else_params
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
                out.push_str(&format!("  %{} = {}\n", v, self.fmt_expr(&val.expr)));
            }

            if let Some(term) = &block.terminator {
                out.push_str(&self.fmt_term(term))
            }

            out.push('\n');
        }

        out
    }
}
