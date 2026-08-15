use std::collections::HashMap;

use crate::ssa::*;
use crate::types::Type;

use thiserror::Error;

#[derive(Debug, Clone, Error)]
pub enum VerifyError {
    #[error("block B{0} is empty (no insts/terminator)")]
    EmptyBlock(BlockId),

    #[error("block B{0} has invalid inst {1}")]
    InvalidInstInBlock(BlockId, InstId),

    #[error("value {0} has invalid def {1}")]
    InvalidValueDef(ValueId, InstId),

    #[error("inst {0} uses undefined value {1}")]
    UndefinedValueUse(InstId, ValueId),

    #[error("inst {0} has invalid result value {1}")]
    InvalidResultValue(InstId, ValueId),

    #[error("value {0} def mismatch: expected {1}, got {2}")]
    ValueDefMismatch(ValueId, InstId, InstId),

    #[error("inst {0:?} expects {1} operands")]
    OperandCountMismatch(InstId, usize),

    #[error("ret inst {0} has invalid operand count")]
    RetOperandMismatch(InstId),

    #[error("ret inst {0} expects {1:?} args, got {2:?}")]
    RetTypeMismatch(InstId, Type, Type),

    #[error("jump to invalid block B{0}")]
    InvalidJumpTarget(BlockId),

    #[error("jump mismatch: B{0} expects {1} args, got {2}")]
    JumpArityMismatch(BlockId, usize, usize),

    #[error("jump type mismatch: B{0} param {1}, expects {2:?} got {3:?}")]
    JumpTypeMismatch(BlockId, usize, Type, Type),

    #[error("use index out of bounds: v{0} in inst {1}")]
    UseIndexOutOfBounds(ValueId, InstId),

    #[error("use mismatch: v{0} in inst {1}")]
    UseMismatch(ValueId, InstId),

    #[error("value {0} has invalid use inst {1}")]
    InvalidUse(ValueId, InstId),

    #[error("inst {0} produces value but has no result")]
    MissingResult(InstId),

    #[error("jump B{0} not listed as successor of B{1}")]
    CFGMissingEdge(BlockId, BlockId),

    #[error("block B{0} has invalid successor B{1}")]
    InvalidSuccessor(BlockId, BlockId),

    #[error("invalid terminator in block B{0}")]
    InvalidTerminator(BlockId),

    #[error("type mismatch: expected {expected:?} got {got:?}")]
    TypeMismatch { expected: Type, got: Type },

    #[error("undeclared function fn{0}")]
    UndeclaredFunction(FuncId),

    #[error("arg count mismatch: fn{0} expects {1} args, got {2}")]
    FuncArgCountMismatch(FuncId, usize, usize),

    #[error("arg type mismatch: fn{0} param {1}, expects {2:?} got {3:?}")]
    FuncArgTypeMismatch(FuncId, usize, Type, Type),

    #[error("bad entry parameters in fn{0}")]
    BadEntryParams(FuncId),
}

impl Module {
    pub fn verify(&self) -> Result<(), HashMap<FuncId, VerifyError>> {
        let mut errs = HashMap::<FuncId, VerifyError>::new();
        for (id, f) in self.iter_functions() {
            if let Err(e) = self.verify_function(id, f) {
                errs.insert(id, e.clone());
                log::error!("verify error in {}: {}", f.name, e);
            }
        }

        if errs.is_empty() {
            Ok(())
        } else {
            Err(errs)
        }
    }

    fn verify_function(&self, _id: FuncId, fun: &Function) -> Result<(), VerifyError> {
        let sig = fun.signature.clone();
        let f = fun.get_definition().unwrap();

        // block verify
        for (bid, block) in f.get_blocks() {
            if block.get_insts().is_empty() && block.get_terminator().is_none() {
                return Err(VerifyError::EmptyBlock(*bid));
            }

            if let Some(&last) = block.get_insts().last() {
                let inst = f
                    .get_insts()
                    .get(&last)
                    .ok_or(VerifyError::InvalidInstInBlock(*bid, last))?;

                if !inst.kind.is_terminator() && block.get_terminator().is_none() {
                    return Err(VerifyError::InvalidTerminator(*bid));
                }
            }
        }

        for (id, inst) in f.get_insts() {
            for &op in &inst.operands {
                if !f.get_values().contains_key(&op) {
                    return Err(VerifyError::UndefinedValueUse(*id, op));
                }
            }

            if let Some(res) = inst.result {
                let val = f
                    .get_values()
                    .get(&res)
                    .ok_or(VerifyError::InvalidResultValue(*id, res))?;

                if val.get_def() != *id {
                    return Err(VerifyError::ValueDefMismatch(res, *id, val.get_def()));
                }
            } else {
                if !inst.kind.is_terminator() && !matches!(inst.kind, InstKind::Store { .. }) {
                    return Err(VerifyError::MissingResult(*id));
                }
            }

            let nops = inst.kind.operand_count();

            #[allow(clippy::collapsible_match)]
            match inst.kind {
                InstKind::Add
                | InstKind::Sub
                | InstKind::Mul
                | InstKind::Div { .. }
                | InstKind::And
                | InstKind::Or
                | InstKind::Xor
                | InstKind::LShl
                | InstKind::LShr
                | InstKind::AShr => {
                    if inst.operands.len() != nops {
                        return Err(VerifyError::OperandCountMismatch(*id, nops));
                    }
                }

                InstKind::Cmp(_) => {
                    if inst.operands.len() != nops {
                        return Err(VerifyError::OperandCountMismatch(*id, nops));
                    }
                }

                InstKind::Ret => {
                    let sret_ty = sig.returns;
                    let op_count = inst.operands.len();

                    match (sret_ty.is_void(), op_count) {
                        // void function must have ret without operands
                        (true, 0) => {}

                        // non-void function must have exactly one operand
                        (false, 1) => {
                            let ret_ty = f.get_type_of(inst.operands[0]);
                            if ret_ty != sret_ty {
                                return Err(VerifyError::RetTypeMismatch(*id, sret_ty, ret_ty));
                            }
                        }

                        _ => {
                            return Err(VerifyError::RetOperandMismatch(*id));
                        }
                    }
                }

                InstKind::Jump(target) => {
                    let target_block = f
                        .get_blocks()
                        .get(&target)
                        .ok_or(VerifyError::InvalidJumpTarget(target))?;

                    let expected = target_block.get_params().len();
                    let actual = inst.operands.len();

                    if expected != actual {
                        return Err(VerifyError::JumpArityMismatch(target, expected, actual));
                    }

                    // type checking
                    for (i, &op) in inst.operands.iter().enumerate() {
                        let op_ty = f.get_type_of(op);

                        let param_id = target_block.get_params()[i];
                        let param_ty = f.get_type_of(param_id);

                        if op_ty != param_ty {
                            return Err(VerifyError::JumpTypeMismatch(target, i, param_ty, op_ty));
                        }
                    }
                }

                InstKind::JumpIf {
                    then_block,
                    else_block,
                } => {
                    let ops = f.get_jumpif_params(inst).unwrap();

                    let then_target = f
                        .get_blocks()
                        .get(&then_block)
                        .ok_or(VerifyError::InvalidJumpTarget(then_block))?;

                    let else_target = f
                        .get_blocks()
                        .get(&else_block)
                        .ok_or(VerifyError::InvalidJumpTarget(else_block))?;

                    let cond_val = inst.operands[0];
                    let cond_ty = f.get_type_of(cond_val);
                    if cond_ty != Type::Int1 {
                        return Err(VerifyError::TypeMismatch {
                            expected: Type::Int1,
                            got: cond_ty,
                        });
                    }

                    let then_param_count = then_target.get_params().len();
                    let else_param_count = else_target.get_params().len();

                    if then_param_count != ops.1.len() {
                        return Err(VerifyError::OperandCountMismatch(*id, then_param_count));
                    }

                    if else_param_count != ops.2.len() {
                        return Err(VerifyError::OperandCountMismatch(*id, else_param_count));
                    }

                    for (i, &op) in ops.1.iter().enumerate() {
                        let op_ty = f.get_type_of(op);
                        let param_id = then_target.get_params()[i];
                        let param_ty = f.get_type_of(param_id);

                        if op_ty != param_ty {
                            return Err(VerifyError::JumpTypeMismatch(
                                then_block, i, param_ty, op_ty,
                            ));
                        }
                    }

                    for (i, &op) in ops.2.iter().enumerate() {
                        let op_ty = f.get_type_of(op);
                        let param_id = else_target.get_params()[i];
                        let param_ty = f.get_type_of(param_id);

                        if op_ty != param_ty {
                            return Err(VerifyError::JumpTypeMismatch(
                                then_block, i, param_ty, op_ty,
                            ));
                        }
                    }
                }

                InstKind::Store { .. } | InstKind::Load { .. } => {
                    let expected = Type::Pointer;
                    let actual = f.get_type_of(inst.operands[0]);
                    if expected != actual {
                        return Err(VerifyError::TypeMismatch {
                            expected,
                            got: actual,
                        });
                    }
                }

                InstKind::Call(fid) => {
                    let func = self
                        .get_function(fid)
                        .ok_or(VerifyError::UndeclaredFunction(fid))?;

                    let expected = func.signature.params.len();
                    let actual = inst.operands.len();

                    if expected != actual {
                        return Err(VerifyError::FuncArgCountMismatch(fid, expected, actual));
                    }

                    for (i, &op) in inst.operands.iter().enumerate() {
                        let op_ty = f.get_type_of(op);

                        let param_ty = func.signature.params[i];

                        if op_ty != param_ty {
                            return Err(VerifyError::FuncArgTypeMismatch(fid, i, param_ty, op_ty));
                        }
                    }
                }

                InstKind::PtrOffset | InstKind::ElementPtr(_) => {
                    if inst.operands.len() != nops {
                        return Err(VerifyError::OperandCountMismatch(*id, nops));
                    }

                    let vbase = inst.operands[0];

                    let basety = f.get_type_of(vbase);
                    if basety != Type::Pointer {
                        return Err(VerifyError::TypeMismatch {
                            expected: Type::Pointer,
                            got: basety,
                        });
                    }
                }

                _ => {
                    if inst.operands.len() != nops {
                        return Err(VerifyError::OperandCountMismatch(*id, nops));
                    }
                }
            }
        }

        // verify values
        for (vid, val) in f.get_values() {
            #[allow(clippy::collapsible_if)]
            if val.get_def() != InstId::MAX {
                if !f.get_insts().contains_key(&val.get_def()) {
                    return Err(VerifyError::InvalidValueDef(*vid, val.get_def()));
                }
            }

            for u in val.get_uses() {
                let inst = f
                    .get_insts()
                    .get(&u.get_inst())
                    .ok_or(VerifyError::InvalidUse(*vid, u.get_index()))?;

                if u.get_index() as usize >= inst.operands.len() {
                    return Err(VerifyError::UseIndexOutOfBounds(*vid, u.get_index()));
                }

                if inst.operands[u.get_index() as usize] != *vid {
                    return Err(VerifyError::UseMismatch(*vid, u.get_inst()));
                }
            }
        }

        // verify cfg
        for (bid, block) in f.get_blocks() {
            for &inst_id in block.get_insts() {
                let inst = f.get_inst(inst_id).unwrap();

                match &inst.kind {
                    InstKind::Jump(target) => {
                        if !f.get_blocks().contains_key(target) {
                            return Err(VerifyError::InvalidJumpTarget(*target));
                        }

                        if !block.get_succs().contains(target) {
                            return Err(VerifyError::CFGMissingEdge(*bid, *target));
                        }
                    }

                    InstKind::JumpIf {
                        then_block,
                        else_block,
                    } => {
                        if !f.get_blocks().contains_key(then_block) {
                            return Err(VerifyError::InvalidJumpTarget(*then_block));
                        }

                        if !block.get_succs().contains(then_block) {
                            return Err(VerifyError::CFGMissingEdge(*bid, *then_block));
                        }

                        if !f.get_blocks().contains_key(else_block) {
                            return Err(VerifyError::InvalidJumpTarget(*else_block));
                        }

                        if !block.get_succs().contains(else_block) {
                            return Err(VerifyError::CFGMissingEdge(*bid, *else_block));
                        }
                    }

                    InstKind::Ret => {}

                    _ => {
                        if inst.kind.is_terminator() {
                            return Err(VerifyError::InvalidTerminator(*bid));
                        }
                    }
                }
            }

            // check preds/succs consistency
            for succ in block.get_succs() {
                if !f.get_blocks().contains_key(succ) {
                    return Err(VerifyError::InvalidSuccessor(*bid, *succ));
                }
            }
        }

        Ok(())
    }
}
