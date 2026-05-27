use std::collections::HashMap;

use crate::tree::*;

impl From<FunctionDef> for crate::ssa::FunctionDef {
    fn from(value: FunctionDef) -> Self {
        let mut dst: crate::ssa::FunctionDef = crate::ssa::FunctionDef::new();
        let mut lowerer = SsaRaiser::new();
        lowerer.raise_function(&value, &mut dst);
        dst
    }
}

#[derive(Debug, Default)]
pub struct SsaRaiser {
    pub value_map: HashMap<ValueId, crate::ssa::ValueId>,
    pub block_map: HashMap<BlockId, crate::ssa::BlockId>,
}

impl SsaRaiser {
    pub fn new() -> Self {
        Self {
            value_map: HashMap::new(),
            block_map: HashMap::new(),
        }
    }

    pub fn raise_function(&mut self, src: &FunctionDef, dst: &mut crate::ssa::FunctionDef) {
        let rpo = src.reverse_post_order();

        for id in rpo.iter() {
            let block = &src.blocks[id];
            let is_entry = src.get_entry() == *id;
            let did = if is_entry { dst.entry } else { dst.new_block() };
            log::info!("entry = {}, did = {}", dst.entry == did, did);
            self.block_map.insert(*id, did);
            for p in &block.params {
                log::info!("block {} : param {}", id, p);
                let ty = src.values[p].ty;
                let v = if is_entry {
                    dst.add_param(ty)
                } else {
                    dst.add_block_param(did, ty)
                };
                self.value_map.insert(*p, v);
            }
        }
        for id in rpo.iter() {
            self.raise_block(*id, src, dst);
        }
    }

    fn raise_block(&mut self, b: BlockId, src: &FunctionDef, dst: &mut crate::ssa::FunctionDef) {
        let dst_block = self.block_map[&b];
        let block = &src.blocks[&b];

        for &vid in &block.values {
            let v = &src.values[&vid];
            let ssa_val = self.raise_expr(v, dst, dst_block);
            self.value_map.insert(vid, ssa_val);
        }

        if let Some(term) = &block.terminator {
            self.raise_term(term, src, dst, dst_block);
        }
    }

    fn raise_term(
        &mut self,
        t: &Terminator,
        _src: &FunctionDef,
        dst: &mut crate::ssa::FunctionDef,
        block: BlockId,
    ) {
        match t {
            Terminator::Ret(v) => {
                let val = self.raise_expr(v, dst, block);
                dst.make_ret(block, Some(val));
            }

            Terminator::RetVoid => {
                dst.make_ret(block, None);
            }

            Terminator::Br { bb, params } => {
                let target = self.block_map[bb];
                let mapped: Vec<_> = params
                    .iter()
                    .map(|p| self.raise_expr(p, dst, block))
                    .collect();
                dst.make_jump(block, target, mapped);
            }

            Terminator::BrIf {
                cond,
                then_bb,
                then_params,
                else_bb,
                else_params,
            } => {
                let cond_v = self.raise_expr(cond, dst, block);
                let t_bb = self.block_map[then_bb];
                let e_bb = self.block_map[else_bb];
                let t_params: Vec<_> = then_params
                    .iter()
                    .map(|p| self.raise_expr(p, dst, block))
                    .collect();
                let e_params: Vec<_> = else_params
                    .iter()
                    .map(|p| self.raise_expr(p, dst, block))
                    .collect();
                dst.make_jumpif(block, cond_v, t_bb, t_params, e_bb, e_params);
            }
        }
    }

    fn raise_expr(
        &mut self,
        e: &Expr,
        dst: &mut crate::ssa::FunctionDef,
        block: BlockId,
    ) -> ValueId {
        log::trace!("lowering expression: {:?}", e);

        let ty = e.ty;

        match &e.kind {
            ExprKind::Var(v) => self.value_map[v],
            ExprKind::Const(c) => dst.make_iconst(block, ty, *c),
            ExprKind::FConst(c) => dst.make_fconst_bits(block, ty, *c),

            ExprKind::Call(func, params) => {
                let params = params
                    .iter()
                    .map(|expr| self.raise_expr(expr, dst, block))
                    .collect();
                dst.make_call(block, ty, *func, params)
            }

            ExprKind::Alloca(ty) => dst.make_alloca(block, *ty),
            ExprKind::NAlloca(ty, cnt) => dst.make_nalloca(block, *ty, *cnt),

            ExprKind::PtrOffset(base, offset) => {
                let base = self.raise_expr(base, dst, block);
                let offset = self.raise_expr(offset, dst, block);
                dst.make_ptr_offset(block, base, offset)
            }

            ExprKind::ElementPtr(elem_ty, base, offset) => {
                let base = self.raise_expr(base, dst, block);
                let offset = self.raise_expr(offset, dst, block);
                dst.make_element_ptr(block, *elem_ty, base, offset)
            }

            ExprKind::Load(volatile, ptr) => {
                let ptr = self.raise_expr(ptr, dst, block);
                dst.make_load(block, *volatile, ty, ptr)
            }

            ExprKind::Store(volatile, ptr, value) => {
                let ptr = self.raise_expr(ptr, dst, block);
                let value = self.raise_expr(value, dst, block);
                dst.make_store(block, *volatile, ptr, value)
            }

            ExprKind::Add(a, b) => {
                let l = self.raise_expr(a, dst, block);
                let r = self.raise_expr(b, dst, block);
                dst.make_add(block, ty, l, r)
            }

            ExprKind::Sub(a, b) => {
                let l = self.raise_expr(a, dst, block);
                let r = self.raise_expr(b, dst, block);
                dst.make_sub(block, ty, l, r)
            }

            ExprKind::Mul(a, b) => {
                let l = self.raise_expr(a, dst, block);
                let r = self.raise_expr(b, dst, block);
                dst.make_mul(block, ty, l, r)
            }

            ExprKind::Div(signed, a, b) => {
                let l = self.raise_expr(a, dst, block);
                let r = self.raise_expr(b, dst, block);
                dst.make_div(block, *signed, ty, l, r)
            }

            ExprKind::Rem(signed, a, b) => {
                let l = self.raise_expr(a, dst, block);
                let r = self.raise_expr(b, dst, block);
                dst.make_rem(block, *signed, ty, l, r)
            }

            ExprKind::FAdd(a, b) => {
                let l = self.raise_expr(a, dst, block);
                let r = self.raise_expr(b, dst, block);
                dst.make_fadd(block, ty, l, r)
            }

            ExprKind::FSub(a, b) => {
                let l = self.raise_expr(a, dst, block);
                let r = self.raise_expr(b, dst, block);
                dst.make_fsub(block, ty, l, r)
            }

            ExprKind::FMul(a, b) => {
                let l = self.raise_expr(a, dst, block);
                let r = self.raise_expr(b, dst, block);
                dst.make_fmul(block, ty, l, r)
            }

            ExprKind::FDiv(a, b) => {
                let l = self.raise_expr(a, dst, block);
                let r = self.raise_expr(b, dst, block);
                dst.make_fdiv(block, ty, l, r)
            }

            ExprKind::FRem(a, b) => {
                let l = self.raise_expr(a, dst, block);
                let r = self.raise_expr(b, dst, block);
                dst.make_frem(block, ty, l, r)
            }

            ExprKind::BitShl(a, b) => {
                let l = self.raise_expr(a, dst, block);
                let r = self.raise_expr(b, dst, block);
                dst.make_lshl(block, ty, l, r)
            }

            ExprKind::BitShr(a, b) => {
                let l = self.raise_expr(a, dst, block);
                let r = self.raise_expr(b, dst, block);
                dst.make_lshr(block, ty, l, r)
            }

            ExprKind::ArithShr(a, b) => {
                let l = self.raise_expr(a, dst, block);
                let r = self.raise_expr(b, dst, block);
                dst.make_ashr(block, ty, l, r)
            }

            ExprKind::BitAnd(a, b) => {
                let l = self.raise_expr(a, dst, block);
                let r = self.raise_expr(b, dst, block);
                dst.make_and(block, ty, l, r)
            }

            ExprKind::BitOr(a, b) => {
                let l = self.raise_expr(a, dst, block);
                let r = self.raise_expr(b, dst, block);
                dst.make_or(block, ty, l, r)
            }

            ExprKind::BitXor(a, b) => {
                let l = self.raise_expr(a, dst, block);
                let r = self.raise_expr(b, dst, block);
                dst.make_xor(block, ty, l, r)
            }

            ExprKind::BitNeg(a) => {
                let v = self.raise_expr(a, dst, block);
                dst.make_not(block, ty, v)
            }

            ExprKind::Cmp(kind, a, b) => {
                let l = self.raise_expr(a, dst, block);
                let r = self.raise_expr(b, dst, block);
                dst.make_cmp(block, *kind, l, r)
            }

            ExprKind::FCmp(kind, a, b) => {
                let l = self.raise_expr(a, dst, block);
                let r = self.raise_expr(b, dst, block);
                dst.make_fcmp(block, *kind, l, r)
            }

            ExprKind::Cast(kind, a) => {
                let l = self.raise_expr(a, dst, block);
                dst.make_cast(block, *kind, ty, l)
            }
        }
    }
}
