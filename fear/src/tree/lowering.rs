use std::collections::HashMap;

use crate::types::Type;

use super::*;

impl From<FunctionDef> for crate::ssa::FunctionDef {
    fn from(value: FunctionDef) -> Self {
        let mut dst: crate::ssa::FunctionDef = crate::ssa::FunctionDef::new();
        let mut lowerer = FearLowerer::new();
        lowerer.lower_function(&value, &mut dst);
        dst
    }
}

#[derive(Debug, Default)]
pub struct FearLowerer {
    pub vmap: HashMap<ValueId, crate::ssa::ValueId>,
    pub bmap: HashMap<BlockId, crate::ssa::BlockId>,
}

impl FearLowerer {
    pub fn new() -> Self {
        Self {
            vmap: HashMap::new(),
            bmap: HashMap::new(),
        }
    }

    pub fn lower_function(&mut self, src: &FunctionDef, dst: &mut crate::ssa::FunctionDef) {
        let rpo = src.reverse_post_order();

        for id in rpo.iter() {
            let block = &src.blocks[id];
            let is_entry = src.get_entry() == *id;
            let did = if is_entry { dst.entry } else { dst.new_block() };
            log::info!("entry = {}, did = {}", dst.entry == did, did);
            self.bmap.insert(*id, did);
            for p in &block.params {
                let ty = src.values[p].ty;
                let v = if is_entry {
                    dst.add_param(ty)
                } else {
                    dst.add_block_param(did, ty)
                };
                self.vmap.insert(*p, v);
            }
        }
        for id in rpo.iter() {
            self.lower_block(*id, src, dst);
        }
    }

    fn lower_block(&mut self, b: BlockId, src: &FunctionDef, dst: &mut crate::ssa::FunctionDef) {
        let dst_block = self.bmap[&b];
        let block = &src.blocks[&b];

        for &vid in &block.values {
            let v = &src.values[&vid];
            let ssa_val = self.lower_expr(&v.expr, dst, dst_block, v.ty);
            self.vmap.insert(vid, ssa_val);
        }

        if let Some(term) = &block.terminator {
            self.lower_terminator(term, src, dst, dst_block);
        }
    }

    fn lower_terminator(
        &mut self,
        t: &Terminator,
        _src: &FunctionDef,
        dst: &mut crate::ssa::FunctionDef,
        block: BlockId,
    ) {
        match t {
            Terminator::Ret(v) => {
                let val = self.vmap[v];
                dst.make_ret(block, Some(val));
            }

            Terminator::Br { bb, params } => {
                let target = self.bmap[bb];
                let mapped: Vec<_> = params.iter().map(|p| self.vmap[p]).collect();
                dst.make_jump(block, target, mapped);
            }

            Terminator::BrIf {
                cond,
                then_bb,
                then_params,
                else_bb,
                else_params,
            } => {
                let cond_v = self.vmap[cond];
                let t_bb = self.bmap[then_bb];
                let e_bb = self.bmap[else_bb];
                let t_params: Vec<_> = then_params.iter().map(|p| self.vmap[p]).collect();
                let e_params: Vec<_> = else_params.iter().map(|p| self.vmap[p]).collect();
                dst.make_jumpif(block, cond_v, t_bb, t_params, e_bb, e_params);
            }
        }
    }

    fn lower_expr(
        &mut self,
        e: &Expr,
        dst: &mut crate::ssa::FunctionDef,
        block: BlockId,
        ty: Type,
    ) -> ValueId {
        log::trace!("lowering expression: {:?}", e);

        match e {
            Expr::Var(v) => self.vmap[v],
            Expr::Const(c) => dst.make_iconst(block, ty, *c),

            Expr::Call(func, params) => {
                let params = params
                    .iter()
                    .map(|expr| self.lower_expr(expr, dst, block, ty))
                    .collect();
                dst.make_call(block, ty, *func, params)
            }

            Expr::Alloca(ty) => dst.make_alloca(block, *ty),

            Expr::PtrOffset(base, offset) => {
                let base = self.lower_expr(base, dst, block, ty);
                let offset = self.lower_expr(offset, dst, block, ty);
                dst.make_ptr_offset(block, base, offset)
            }

            Expr::ElementPtr(elem_ty, base, offset) => {
                let base = self.lower_expr(base, dst, block, ty);
                let offset = self.lower_expr(offset, dst, block, ty);
                dst.make_element_ptr(block, *elem_ty, base, offset)
            }

            Expr::Load(volatile, ptr) => {
                let ptr = self.lower_expr(ptr, dst, block, ty);
                dst.make_load(block, *volatile, ty, ptr)
            }

            Expr::Store(volatile, ptr, value) => {
                let ptr = self.lower_expr(ptr, dst, block, ty);
                let value = self.lower_expr(value, dst, block, ty);
                dst.make_store(block, *volatile, ptr, value)
            }

            Expr::Add(a, b) => {
                let l = self.lower_expr(a, dst, block, ty);
                let r = self.lower_expr(b, dst, block, ty);
                dst.make_add(block, ty, l, r)
            }

            Expr::Sub(a, b) => {
                let l = self.lower_expr(a, dst, block, ty);
                let r = self.lower_expr(b, dst, block, ty);
                dst.make_sub(block, ty, l, r)
            }

            Expr::Mul(a, b) => {
                let l = self.lower_expr(a, dst, block, ty);
                let r = self.lower_expr(b, dst, block, ty);
                dst.make_mul(block, ty, l, r)
            }

            Expr::Div(signed, a, b) => {
                let l = self.lower_expr(a, dst, block, ty);
                let r = self.lower_expr(b, dst, block, ty);
                dst.make_div(block, *signed, ty, l, r)
            }

            Expr::BitShl(a, b) => {
                let l = self.lower_expr(a, dst, block, ty);
                let r = self.lower_expr(b, dst, block, ty);
                dst.make_lshl(block, ty, l, r)
            }

            Expr::BitShr(a, b) => {
                let l = self.lower_expr(a, dst, block, ty);
                let r = self.lower_expr(b, dst, block, ty);
                dst.make_lshr(block, ty, l, r)
            }

            Expr::ArithShr(a, b) => {
                let l = self.lower_expr(a, dst, block, ty);
                let r = self.lower_expr(b, dst, block, ty);
                dst.make_ashr(block, ty, l, r)
            }

            Expr::BitAnd(a, b) => {
                let l = self.lower_expr(a, dst, block, ty);
                let r = self.lower_expr(b, dst, block, ty);
                dst.make_and(block, ty, l, r)
            }

            Expr::BitOr(a, b) => {
                let l = self.lower_expr(a, dst, block, ty);
                let r = self.lower_expr(b, dst, block, ty);
                dst.make_or(block, ty, l, r)
            }

            Expr::BitXor(a, b) => {
                let l = self.lower_expr(a, dst, block, ty);
                let r = self.lower_expr(b, dst, block, ty);
                dst.make_xor(block, ty, l, r)
            }

            Expr::BitNeg(a) => {
                let v = self.lower_expr(a, dst, block, ty);
                dst.make_not(block, ty, v)
            }

            Expr::Cmp(kind, a, b) => {
                let l = self.lower_expr(a, dst, block, ty);
                let r = self.lower_expr(b, dst, block, ty);
                dst.make_cmp(block, *kind, l, r)
            }

            Expr::FCmp(kind, a, b) => {
                let l = self.lower_expr(a, dst, block, ty);
                let r = self.lower_expr(b, dst, block, ty);
                dst.make_fcmp(block, *kind, l, r)
            }
        }
    }
}
