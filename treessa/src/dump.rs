use crate::*;

impl FunctionDef {
    pub fn fmt_expr(expr: &Expr) -> String {
        match expr {
            Expr::Var(v) => format!("%{}", v),
            Expr::Const(c) => c.to_string(),

            Expr::Add(a, b) => format!("add({}, {})", Self::fmt_expr(a), Self::fmt_expr(b)),
            Expr::Sub(a, b) => format!("sub({}, {})", Self::fmt_expr(a), Self::fmt_expr(b)),
            Expr::Mul(a, b) => format!("mul({}, {})", Self::fmt_expr(a), Self::fmt_expr(b)),
            Expr::Div(a, b) => format!("div({}, {})", Self::fmt_expr(a), Self::fmt_expr(b)),
            Expr::BitShl(a, b) => format!("shl({}, {})", Self::fmt_expr(a), Self::fmt_expr(b)),
            Expr::BitShr(a, b) => format!("shr({}, {})", Self::fmt_expr(a), Self::fmt_expr(b)),
            Expr::ArithShr(a, b) => format!("ashr({}, {})", Self::fmt_expr(a), Self::fmt_expr(b)),
            Expr::BitAnd(a, b) => format!("band({}, {})", Self::fmt_expr(a), Self::fmt_expr(b)),
            Expr::BitOr(a, b) => format!("bor({}, {})", Self::fmt_expr(a), Self::fmt_expr(b)),
            Expr::BitXor(a, b) => format!("bxor({}, {})", Self::fmt_expr(a), Self::fmt_expr(b)),
            Expr::BitNeg(a) => format!("bneg({})", Self::fmt_expr(a)),
        }
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
                out.push_str(&format!("  %{} = {}\n", v, Self::fmt_expr(&val.expr)));
            }

            if let Some(Terminator::Ret(v)) = block.terminator {
                let val = &self.values[&v];
                out.push_str(&format!("  ret {} %{}\n", val.ty, v));
            }

            out.push('\n');
        }

        out
    }
}
