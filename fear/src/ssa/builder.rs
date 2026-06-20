use crate::types::{CastKind, FloatCmp, IntCmp, Type};

use super::ir::*;

impl FunctionDef {
    /// Makes integer constant value
    pub fn make_int_const(&mut self, block: BlockId, ty: Type, x: i64) -> ValueId {
        self.append_inst(block, InstKind::IConst(x), ty, vec![])
    }

    /// Makes float constant value from raw bits
    pub fn make_float_const_from_bits(&mut self, block: BlockId, ty: Type, bits: u64) -> ValueId {
        self.append_inst(block, InstKind::FConst(bits), ty, vec![])
    }

    /// Make float constant value
    pub fn make_float_const(&mut self, block: BlockId, ty: Type, x: f64) -> ValueId {
        self.append_inst(block, InstKind::FConst(x.to_bits()), ty, vec![])
    }

    pub fn make_undef(&mut self, block: BlockId, ty: Type) -> ValueId {
        self.append_inst(block, InstKind::Undef, ty, vec![])
    }

    pub fn make_select(
        &mut self,
        block: BlockId,
        ty: Type,
        cond: ValueId,
        then_value: ValueId,
        else_value: ValueId,
    ) -> ValueId {
        self.append_inst(
            block,
            InstKind::Select,
            ty,
            vec![cond, then_value, else_value],
        )
    }

    fn make_unary(&mut self, block: BlockId, kind: InstKind, ty: Type, value: ValueId) -> ValueId {
        self.append_inst(block, kind, ty, vec![value])
    }

    fn make_binary(
        &mut self,
        block: BlockId,
        kind: InstKind,
        ty: Type,
        lhs: ValueId,
        rhs: ValueId,
    ) -> ValueId {
        self.append_inst(block, kind, ty, vec![lhs, rhs])
    }

    pub fn make_int_cmp(
        &mut self,
        block: BlockId,
        kind: IntCmp,
        lhs: ValueId,
        rhs: ValueId,
    ) -> ValueId {
        self.make_binary(block, InstKind::Cmp(kind), Type::Int1, lhs, rhs)
    }

    pub fn make_float_cmp(
        &mut self,
        block: BlockId,
        kind: FloatCmp,
        lhs: ValueId,
        rhs: ValueId,
    ) -> ValueId {
        self.make_binary(block, InstKind::FCmp(kind), Type::Int1, lhs, rhs)
    }

    pub fn make_cast(
        &mut self,
        block: BlockId,
        kind: CastKind,
        ty: Type,
        value: ValueId,
    ) -> ValueId {
        self.make_unary(block, InstKind::Cast(kind), ty, value)
    }

    pub fn make_int_add(
        &mut self,
        block: BlockId,
        ty: Type,
        lhs: ValueId,
        rhs: ValueId,
    ) -> ValueId {
        self.make_binary(block, InstKind::Add, ty, lhs, rhs)
    }

    pub fn make_int_sub(
        &mut self,
        block: BlockId,
        ty: Type,
        lhs: ValueId,
        rhs: ValueId,
    ) -> ValueId {
        self.make_binary(block, InstKind::Sub, ty, lhs, rhs)
    }

    pub fn make_int_mul(
        &mut self,
        block: BlockId,
        ty: Type,
        lhs: ValueId,
        rhs: ValueId,
    ) -> ValueId {
        self.make_binary(block, InstKind::Mul, ty, lhs, rhs)
    }

    pub fn make_int_div(
        &mut self,
        block: BlockId,
        signed: bool,
        ty: Type,
        lhs: ValueId,
        rhs: ValueId,
    ) -> ValueId {
        self.make_binary(block, InstKind::Div { signed }, ty, lhs, rhs)
    }

    pub fn make_int_rem(
        &mut self,
        block: BlockId,
        signed: bool,
        ty: Type,
        lhs: ValueId,
        rhs: ValueId,
    ) -> ValueId {
        self.make_binary(block, InstKind::Rem { signed }, ty, lhs, rhs)
    }

    pub fn make_int_neg(&mut self, block: BlockId, ty: Type, value: ValueId) -> ValueId {
        self.make_unary(block, InstKind::Neg, ty, value)
    }

    pub fn make_float_add(
        &mut self,
        block: BlockId,
        ty: Type,
        lhs: ValueId,
        rhs: ValueId,
    ) -> ValueId {
        self.make_binary(block, InstKind::FAdd, ty, lhs, rhs)
    }

    pub fn make_float_sub(
        &mut self,
        block: BlockId,
        ty: Type,
        lhs: ValueId,
        rhs: ValueId,
    ) -> ValueId {
        self.make_binary(block, InstKind::FSub, ty, lhs, rhs)
    }

    pub fn make_float_mul(
        &mut self,
        block: BlockId,
        ty: Type,
        lhs: ValueId,
        rhs: ValueId,
    ) -> ValueId {
        self.make_binary(block, InstKind::FMul, ty, lhs, rhs)
    }

    pub fn make_float_div(
        &mut self,
        block: BlockId,
        ty: Type,
        lhs: ValueId,
        rhs: ValueId,
    ) -> ValueId {
        self.make_binary(block, InstKind::FDiv, ty, lhs, rhs)
    }

    pub fn make_float_rem(
        &mut self,
        block: BlockId,
        ty: Type,
        lhs: ValueId,
        rhs: ValueId,
    ) -> ValueId {
        self.make_binary(block, InstKind::FRem, ty, lhs, rhs)
    }

    pub fn make_float_neg(&mut self, block: BlockId, ty: Type, value: ValueId) -> ValueId {
        self.make_unary(block, InstKind::FNeg, ty, value)
    }

    pub fn make_bitnot(&mut self, block: BlockId, ty: Type, value: ValueId) -> ValueId {
        self.make_unary(block, InstKind::Not, ty, value)
    }

    pub fn make_bitand(&mut self, block: BlockId, ty: Type, lhs: ValueId, rhs: ValueId) -> ValueId {
        self.make_binary(block, InstKind::And, ty, lhs, rhs)
    }

    pub fn make_bitor(&mut self, block: BlockId, ty: Type, lhs: ValueId, rhs: ValueId) -> ValueId {
        self.make_binary(block, InstKind::Or, ty, lhs, rhs)
    }

    pub fn make_bitxor(&mut self, block: BlockId, ty: Type, lhs: ValueId, rhs: ValueId) -> ValueId {
        self.make_binary(block, InstKind::Xor, ty, lhs, rhs)
    }

    /// Makes logical shift left
    pub fn make_lshl(&mut self, block: BlockId, ty: Type, lhs: ValueId, rhs: ValueId) -> ValueId {
        self.make_binary(block, InstKind::LShl, ty, lhs, rhs)
    }

    /// Makes logical shift right
    pub fn make_lshr(&mut self, block: BlockId, ty: Type, lhs: ValueId, rhs: ValueId) -> ValueId {
        self.make_binary(block, InstKind::LShr, ty, lhs, rhs)
    }

    /// Makes arithmetic shift right
    pub fn make_ashr(&mut self, block: BlockId, ty: Type, lhs: ValueId, rhs: ValueId) -> ValueId {
        self.make_binary(block, InstKind::AShr, ty, lhs, rhs)
    }

    pub fn make_store(
        &mut self,
        block: BlockId,
        volatile: bool,
        ptr: ValueId,
        value: ValueId,
    ) -> ValueId {
        self.append_inst(
            block,
            InstKind::Store { volatile },
            Type::Void,
            vec![ptr, value],
        )
    }

    pub fn make_load(&mut self, block: BlockId, volatile: bool, ty: Type, ptr: ValueId) -> ValueId {
        self.append_inst(block, InstKind::Load { volatile }, ty, vec![ptr])
    }

    pub fn make_ptr_offset(&mut self, block: BlockId, base: ValueId, offset: ValueId) -> ValueId {
        self.append_inst(
            block,
            InstKind::PtrOffset,
            Type::Pointer,
            vec![base, offset],
        )
    }

    pub fn make_element_ptr(
        &mut self,
        block: BlockId,
        ty: Type,
        base: ValueId,
        offset: ValueId,
    ) -> ValueId {
        self.append_inst(
            block,
            InstKind::ElementPtr(ty),
            Type::Pointer,
            vec![base, offset],
        )
    }

    pub fn make_alloca(&mut self, block: BlockId, ty: Type) -> ValueId {
        self.append_inst(block, InstKind::Alloca(ty), Type::Pointer, vec![])
    }

    pub fn make_nalloca(&mut self, block: BlockId, ty: Type, count: usize) -> ValueId {
        self.append_inst(block, InstKind::NAlloca(ty, count), Type::Pointer, vec![])
    }

    pub fn make_call(
        &mut self,
        block: BlockId,
        ty: Type,
        func: FuncId,
        operands: Vec<ValueId>,
    ) -> ValueId {
        self.append_inst(block, InstKind::Call(func), ty, operands)
    }

    pub fn make_ret(&mut self, block: BlockId, value: Option<ValueId>) {
        let mut operands = vec![];
        if let Some(v) = value {
            operands.push(v);
        }
        let (i, _) = self.append_inst_base(block, InstKind::Ret, Type::Void, operands);
        self.set_terminator(block, i);
    }

    pub fn make_jump(&mut self, block: BlockId, target: BlockId, params: Vec<ValueId>) {
        let (i, _) = self.append_inst_base(block, InstKind::Jump(target), Type::Void, params);
        self.set_terminator(block, i);
    }

    pub fn make_jumpif(
        &mut self,
        block: BlockId,
        cond: ValueId,
        then_block: BlockId,
        then_params: Vec<ValueId>,
        else_block: BlockId,
        else_params: Vec<ValueId>,
    ) {
        let mut operands = Vec::with_capacity(1 + then_params.len() + else_params.len());

        operands.push(cond);
        operands.extend(then_params);
        operands.extend(else_params);

        let (i, _) = self.append_inst_base(
            block,
            InstKind::JumpIf {
                then_block,
                else_block,
            },
            Type::Void,
            operands,
        );
        self.set_terminator(block, i);
    }
}
