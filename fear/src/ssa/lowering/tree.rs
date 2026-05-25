use std::collections::HashMap;

use crate::ssa;
use crate::tree::*;
use crate::types::Type;

impl From<ssa::FunctionDef> for FunctionDef {
    fn from(src: ssa::FunctionDef) -> Self {
        let mut dst = FunctionDef::new();
        let mut raiser = TreeSsaRaiser::new();
        raiser.raise_function(&src, &mut dst);
        dst
    }
}

#[derive(Debug, Default)]
pub struct TreeSsaRaiser {
    value_map: HashMap<ssa::ValueId, ValueId>,
    block_map: HashMap<ssa::BlockId, BlockId>,
}

impl TreeSsaRaiser {
    pub fn new() -> Self {
        Self {
            value_map: HashMap::new(),
            block_map: HashMap::new(),
        }
    }

    pub fn raise_function(&mut self, src: &ssa::FunctionDef, dst: &mut FunctionDef) {
        let rpo = src.reverse_post_order();

        for &bid in &rpo {
            let block = &src.blocks[&bid];
            let is_entry = bid == src.entry;
            let dst_bid = if is_entry {
                dst.get_entry()
            } else {
                dst.new_block()
            };
            self.block_map.insert(bid, dst_bid);
            for &param in &block.params {
                let ty = src.values[&param].ty;
                let var_expr = dst.add_block_param(dst_bid, ty);
                if let ExprKind::Var(v) = &var_expr.kind
                {
                    self.value_map.insert(param, *v);
                }
            }
        }

        for &bid in &rpo {
            self.raise_block(bid, src, dst);
        }
    }

    fn raise_block(&mut self, bid: ssa::BlockId, src: &ssa::FunctionDef, dst: &mut FunctionDef) {
        let dst_bid = self.block_map[&bid];
        let block = &src.blocks[&bid];

        for &iid in &block.insts {
            let inst = &src.insts[&iid];
            if inst.kind.is_terminator() {
                continue;
            }

            let expr = self.raise_inst(src, inst, dst_bid, dst);
            if let Some(result) = inst.result && let ExprKind::Var(v) = &expr.kind {
                self.value_map.insert(result, *v);
            }
        }

        if let Some(term_id) = block.term {
            let inst = &src.insts[&term_id];
            self.raise_terminator(inst, src, dst, dst_bid);
        }
    }

    fn v(&self, ssa_val: ssa::ValueId, dst: &FunctionDef) -> Expr {
        let tvalue = self.value_map[&ssa_val];
        Expr {
            ty: dst.values[&tvalue].ty,
            kind: ExprKind::Var(tvalue),
        }
    }

    fn raise_inst(
        &self,
        src: &ssa::FunctionDef,
        inst: &ssa::Inst,
        block: BlockId,
        dst: &mut FunctionDef,
    ) -> Expr {
        let ops = &inst.operands;
        let ty = if let Some(r) = inst.result {
            src.get_type(r)
        } else {
            Type::Void
        };

        match &inst.kind {
            ssa::InstKind::IConst(c) => {
                dst.make_iconst(block, src.get_type(inst.result.unwrap()), *c)
            }
            ssa::InstKind::FConst(_) => todo!("FConst not in fear Expr"),

            ssa::InstKind::Add => dst.make_add(block, ty, &self.v(ops[0], dst), &self.v(ops[1], dst)),
            ssa::InstKind::Sub => dst.make_sub(block, ty, &self.v(ops[0], dst), &self.v(ops[1], dst)),
            ssa::InstKind::Mul => dst.make_mul(block, ty, &self.v(ops[0], dst), &self.v(ops[1], dst)),
            ssa::InstKind::Div { signed } => {
                dst.make_div(block, ty, *signed, &self.v(ops[0], dst), &self.v(ops[1], dst))
            }
            ssa::InstKind::Rem { signed } => {
                dst.make_rem(block, ty, *signed, &self.v(ops[0], dst), &self.v(ops[1], dst))
            }

            ssa::InstKind::LShl => {
                dst.make_shl(block, ty, &self.v(ops[0], dst), &self.v(ops[1], dst))
            }
            ssa::InstKind::LShr => {
                dst.make_shr(block, ty, &self.v(ops[0], dst), &self.v(ops[1], dst))
            }
            ssa::InstKind::AShr => {
                dst.make_ashr(block, ty, &self.v(ops[0], dst), &self.v(ops[1], dst))
            }

            ssa::InstKind::Not => dst.make_bitneg(block, ty, &self.v(ops[0], dst)),
            ssa::InstKind::And => {
                dst.make_bitand(block, ty, &self.v(ops[0], dst), &self.v(ops[1], dst))
            }
            ssa::InstKind::Or => {
                dst.make_bitor(block, ty, &self.v(ops[0], dst), &self.v(ops[1], dst))
            }
            ssa::InstKind::Xor => {
                dst.make_bitxor(block, ty, &self.v(ops[0], dst), &self.v(ops[1], dst))
            }

            ssa::InstKind::Cmp(kind) => {
                dst.make_icmp(block, *kind, &self.v(ops[0], dst), &self.v(ops[1], dst))
            }
            ssa::InstKind::FCmp(kind) => {
                dst.make_fcmp(block, *kind, &self.v(ops[0], dst), &self.v(ops[1], dst))
            }

            ssa::InstKind::Alloca(ty) => dst.make_alloca(block, *ty),
            ssa::InstKind::NAlloca(ty, cnt) => dst.make_nalloca(block, *ty, *cnt),

            ssa::InstKind::Load { volatile } => {
                dst.make_load(block, ty, *volatile, &self.v(ops[0], dst))
            }
            ssa::InstKind::Store { volatile } => {
                dst.make_store(block, *volatile, &self.v(ops[0], dst), &self.v(ops[1], dst))
            }

            ssa::InstKind::PtrOffset => {
                dst.make_ptr_offset(block, &self.v(ops[0], dst), &self.v(ops[1], dst))
            }
            ssa::InstKind::ElementPtr(elem_ty) => {
                dst.make_element_ptr(block, *elem_ty, &self.v(ops[0], dst), &self.v(ops[1], dst))
            }

            ssa::InstKind::Call(func_id) => {
                let args = ops.iter().map(|&op| self.v(op, dst)).collect();
                dst.make_call(block, ty, *func_id, args)
            }

            ssa::InstKind::Cast(_) => todo!("Cast has no fear Expr equivalent"),

            ssa::InstKind::Ret
            | ssa::InstKind::Jump(_)
            | ssa::InstKind::JumpIf { .. }
            | ssa::InstKind::FAdd
            | ssa::InstKind::FSub
            | ssa::InstKind::FMul
            | ssa::InstKind::FDiv
            | ssa::InstKind::FRem => todo!("{:?} has no fear equivalent", inst.kind),
        }
    }

    fn raise_terminator(
        &self,
        inst: &ssa::Inst,
        src: &ssa::FunctionDef,
        dst: &mut FunctionDef,
        dst_block: BlockId,
    ) {
        match &inst.kind {
            ssa::InstKind::Ret => {
                if inst.operands.is_empty() {
                    dst.make_retvoid(dst_block);
                } else {
                    let expr = self.v(inst.operands[0], dst);
                    dst.make_ret(dst_block, &expr);
                }
            }

            ssa::InstKind::Jump(target) => {
                let dst_target = self.block_map[target];
                let params = inst.operands.iter().map(|&p| self.v(p, dst)).collect();
                dst.make_br(dst_block, dst_target, params);
            }

            ssa::InstKind::JumpIf {
                then_block,
                else_block,
            } => {
                let (cond, then_params, else_params) = src.get_jumpif_params(inst).unwrap();

                let dst_then = self.block_map[then_block];
                let dst_else = self.block_map[else_block];
                let cond_v = self.v(cond, dst);
                let t_params: Vec<_> = then_params.iter().map(|&p| self.v(p, dst)).collect();
                let e_params: Vec<_> = else_params.iter().map(|&p| self.v(p, dst)).collect();
                dst.make_brif(dst_block, &cond_v, dst_then, t_params, dst_else, e_params);
            }

            _ => unreachable!("not a terminator"),
        }
    }
}
