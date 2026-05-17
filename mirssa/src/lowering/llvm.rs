use inkwell::{
    builder::Builder,
    context::Context,
    module::Module,
    types::{AnyTypeEnum, BasicMetadataTypeEnum, BasicTypeEnum, FloatType, FunctionType, IntType},
    values::{BasicValue, BasicValueEnum, FloatValue, FunctionValue, IntValue, PhiValue},
    AddressSpace, IntPredicate,
};

use std::collections::HashMap;

use quasar::{FunctionSignature, Linkage, Type};

use crate::ir::*;

pub struct LlvmLowerer<'ctx> {
    pub context: &'ctx Context,
    pub module: Module<'ctx>,
    pub builder: Builder<'ctx>,

    pub values: HashMap<ValueId, BasicValueEnum<'ctx>>,
    pub blocks: HashMap<BlockId, inkwell::basic_block::BasicBlock<'ctx>>,
    pub phis: HashMap<ValueId, PhiValue<'ctx>>,
    pub functions: HashMap<FuncId, FunctionValue<'ctx>>,
}

impl<'ctx> LlvmLowerer<'ctx> {
    pub fn new(ctx: &'ctx Context, name: &str) -> Self {
        let module = ctx.create_module(name);
        let builder = ctx.create_builder();

        Self {
            context: ctx,
            module,
            builder,
            values: HashMap::new(),
            blocks: HashMap::new(),
            phis: HashMap::new(),
            functions: HashMap::new(),
        }
    }

    // ---------------------------
    // TYPE MAPPING
    // ---------------------------

    pub fn map_type(&self, ty: Type) -> BasicTypeEnum<'ctx> {
        let ctx = self.context;

        match ty {
            Type::I1 => ctx.bool_type().into(),
            Type::I8 => ctx.i8_type().into(),
            Type::I16 => ctx.i16_type().into(),
            Type::I32 => ctx.i32_type().into(),
            Type::I64 => ctx.i64_type().into(),

            Type::F32 => ctx.f32_type().into(),
            Type::F64 => ctx.f64_type().into(),

            Type::Ptr => ctx.ptr_type(AddressSpace::default()).into(),

            Type::Void => panic!("void cannot be BasicType"),
        }
    }

    pub fn map_any_type(&self, ty: Type) -> AnyTypeEnum<'ctx> {
        let ctx = self.context;

        match ty {
            Type::I1 => ctx.bool_type().into(),
            Type::I8 => ctx.i8_type().into(),
            Type::I16 => ctx.i16_type().into(),
            Type::I32 => ctx.i32_type().into(),
            Type::I64 => ctx.i64_type().into(),

            Type::F32 => ctx.f32_type().into(),
            Type::F64 => ctx.f64_type().into(),

            Type::Ptr => ctx.ptr_type(AddressSpace::default()).into(),

            Type::Void => ctx.void_type().into(),
        }
    }

    pub fn map_int_type(&self, ty: Type) -> IntType<'ctx> {
        let ctx = self.context;

        match ty {
            Type::I1 => ctx.bool_type(),
            Type::I8 => ctx.i8_type(),
            Type::I16 => ctx.i16_type(),
            Type::I32 => ctx.i32_type(),
            Type::I64 => ctx.i64_type(),
            _ => panic!("not int"),
        }
    }

    pub fn map_float_type(&self, ty: Type) -> FloatType<'ctx> {
        let ctx = self.context;

        match ty {
            Type::F32 => ctx.f32_type(),
            Type::F64 => ctx.f64_type(),
            _ => panic!("not float"),
        }
    }

    fn map_param_types(&self, params: &[Type]) -> Vec<BasicMetadataTypeEnum<'ctx>> {
        params.iter().map(|ty| self.map_type(*ty).into()).collect()
    }

    fn map_fn_type(&self, returns: Type, params: &[Type]) -> FunctionType<'ctx> {
        let ret_ty = self.map_any_type(returns);
        let params_types: Vec<_> = self.map_param_types(params);

        match ret_ty {
            AnyTypeEnum::IntType(t) => t.fn_type(&params_types, false),
            AnyTypeEnum::FloatType(t) => t.fn_type(&params_types, false),
            AnyTypeEnum::PointerType(t) => t.fn_type(&params_types, false),
            AnyTypeEnum::VoidType(t) => t.fn_type(&params_types, false),

            _ => unreachable!(),
        }
    }

    fn map_signature_type(&self, sig: &FunctionSignature) -> FunctionType<'ctx> {
        self.map_fn_type(sig.returns, &sig.params)
    }

    fn map_linkage(&self, l: &Linkage) -> inkwell::module::Linkage {
        match l {
            Linkage::External => inkwell::module::Linkage::External,
            Linkage::Internal => inkwell::module::Linkage::Internal,
            Linkage::Weak => inkwell::module::Linkage::ExternalWeak,
        }
    }

    fn map_int_pred(&self, kind: CmpKind) -> IntPredicate {
        match kind {
            CmpKind::Eq => IntPredicate::EQ,
            CmpKind::Ne => IntPredicate::NE,
            CmpKind::Lt => IntPredicate::SLT,
            CmpKind::Le => IntPredicate::SLE,
            CmpKind::Gt => IntPredicate::SGT,
            CmpKind::Ge => IntPredicate::SGE,
            CmpKind::ULt => IntPredicate::ULT,
            CmpKind::ULe => IntPredicate::ULE,
            CmpKind::UGt => IntPredicate::UGT,
            CmpKind::UGe => IntPredicate::UGE,
        }
    }

    // ---------------------------
    // ENTRY
    // ---------------------------

    pub fn get_module(&self) -> &inkwell::module::Module<'ctx> {
        &self.module
    }

    pub fn lower_module(&mut self, m: &crate::ir::Module) {
        for (fid, func) in m.iter_functions() {
            log::debug!("declaring function '{}' (fid={})", func.name, fid);
            self.declare_function(m, fid);
        }

        for (fid, func) in m.iter_functions() {
            if func.get_definition().is_some() {
                log::debug!("lowering function '{}' (fid={})", func.name, fid);
                self.lower_function(m, fid);
            }
        }
    }

    pub fn lower_function(&mut self, m: &crate::ir::Module, fid: FuncId) {
        let func = m.get_function(fid).unwrap();
        let def = func.get_definition().unwrap();
        let llvm_fn = self.functions[&fid];

        self.values.clear();
        self.blocks.clear();
        self.phis.clear();

        let entry = def.entry;
        let bb_entry = self
            .context
            .append_basic_block(llvm_fn, &format!("b{}", entry));
        self.blocks.insert(entry, bb_entry);

        for (&bid, _) in &def.blocks {
            if bid != entry {
                let bb = self
                    .context
                    .append_basic_block(llvm_fn, &format!("b{}", bid));
                self.blocks.insert(bid, bb);
            }
        }

        for (i, &param_val) in def.blocks[&entry].params.iter().enumerate() {
            let arg = llvm_fn
                .get_nth_param(i as u32)
                .unwrap_or_else(|| panic!("function '{}' missing param #{}", func.name, i));
            self.values.insert(param_val, arg);
        }

        self.build_phi_nodes(def);

        let block_ids: Vec<BlockId> = def.reverse_post_order();
        for bid in block_ids {
            self.compile_block(m, def, bid);
        }

        let block_ids: Vec<BlockId> = def.blocks.keys().copied().collect();
        for bid in block_ids {
            self.fill_phi_incoming(def, bid);
        }
    }

    pub fn declare_function(&mut self, m: &crate::ir::Module, fid: FuncId) {
        let func = m.get_function(fid).unwrap();
        let fn_ty = self.map_signature_type(&func.signature);
        let llvm_fn =
            self.module
                .add_function(&func.name, fn_ty, Some(self.map_linkage(&func.linkage)));
        self.functions.insert(fid, llvm_fn);
    }

    // ---------------------------
    // PHI LOWERING
    // ---------------------------

    fn build_phi_nodes(&mut self, def: &FunctionDef) {
        for (&bid, block) in &def.blocks {
            if bid == def.entry {
                continue;
            }

            let llvm_bb = self.blocks[&bid];
            self.builder.position_at_end(llvm_bb);

            for &param in &block.params {
                let ty = def.get_type(param);
                let llvm_ty = self.map_type(ty);
                let phi = self.builder.build_phi(llvm_ty, "param").unwrap();
                self.phis.insert(param, phi);
                self.values.insert(param, phi.as_basic_value());
            }
        }
    }

    // ---------------------------
    // BLOCK COMPILATION
    // ---------------------------

    fn fill_phi_incoming(&mut self, def: &FunctionDef, bid: BlockId) {
        if bid == def.entry {
            return; // entry params are function args
        }

        let block = &def.blocks[&bid];

        for (param_idx, &param_val) in block.params.iter().enumerate() {
            let phi = match self.phis.get(&param_val) {
                Some(p) => *p,
                None => continue,
            };

            for &pred_bid in &block.preds {
                let pred_block = &def.blocks[&pred_bid];
                let pred_bb = self.blocks[&pred_bid];

                // Find the terminator of the predecessor.
                let term_id = match pred_block.term {
                    Some(t) => t,
                    None => continue,
                };
                let term = &def.insts[&term_id];

                let incoming_val: BasicValueEnum = match &term.kind {
                    InstKind::Jump(_) => {
                        // operands[param_idx] is the value passed to the target
                        let op = term.operands[param_idx];
                        self.get_or_const(def, op)
                    }

                    InstKind::JumpIf {
                        then_block,
                        else_block,
                    } => {
                        // JumpIf operand layout:
                        //   [cond, then_params..., else_params...]
                        let (_, then_params, else_params) = def.get_jumpif_params(term).unwrap();

                        if *then_block == bid {
                            let op = then_params[param_idx];
                            self.get_or_const(def, op)
                        } else if *else_block == bid {
                            let op = else_params[param_idx];
                            self.get_or_const(def, op)
                        } else {
                            panic!(
                                "block B{} is a predecessor of B{} but neither then nor else",
                                pred_bid, bid
                            );
                        }
                    }

                    other => panic!(
                        "unexpected terminator kind {:?} in predecessor B{}",
                        other, pred_bid
                    ),
                };

                phi.add_incoming(&[(&incoming_val, pred_bb)]);
            }
        }
    }

    fn compile_block(&mut self, m: &crate::ir::Module, def: &FunctionDef, bid: BlockId) {
        let llvm_bb = self.blocks[&bid];
        self.builder.position_at_end(llvm_bb);

        let block = &def.blocks[&bid];

        for &inst_id in &block.insts {
            let inst = &def.insts[&inst_id];
            self.compile_inst(m, def, inst);
        }
    }

    fn compile_inst(&mut self, _m: &crate::ir::Module, def: &FunctionDef, inst: &Inst) {
        log::trace!(
            "lowering inst: {:?}, operands: {:?}",
            inst.kind,
            inst.operands
        );
        match &inst.kind {
            InstKind::IConst(x) => {
                let ty = def.get_type(inst.result.unwrap());
                let val = self.map_int_type(ty).const_int(*x as u64, true);
                self.values.insert(inst.result.unwrap(), val.into());
            }

            InstKind::FConst(bits) => {
                let ty = def.get_type(inst.result.unwrap());
                let val: BasicValueEnum = match ty {
                    Type::F32 => {
                        let f = f32::from_bits(*bits as u32);
                        self.context.f32_type().const_float(f as f64).into()
                    }
                    Type::F64 => {
                        let f = f64::from_bits(*bits);
                        self.context.f64_type().const_float(f).into()
                    }
                    _ => panic!("FConst with non-float type"),
                };
                self.values.insert(inst.result.unwrap(), val);
            }

            // ------------------------------------------------------------------
            // Integer arithmetic
            // ------------------------------------------------------------------
            InstKind::Add => {
                self.bin_int(def, inst, |b, l, r| b.build_int_add(l, r, "add").unwrap())
            }
            InstKind::Sub => {
                self.bin_int(def, inst, |b, l, r| b.build_int_sub(l, r, "sub").unwrap())
            }
            InstKind::Mul => {
                self.bin_int(def, inst, |b, l, r| b.build_int_mul(l, r, "mul").unwrap())
            }
            InstKind::Div { signed } => {
                let s = *signed;
                self.bin_int(def, inst, move |b, l, r| {
                    if s {
                        b.build_int_signed_div(l, r, "sdiv").unwrap()
                    } else {
                        b.build_int_unsigned_div(l, r, "udiv").unwrap()
                    }
                });
            }
            InstKind::Rem { signed } => {
                let s = *signed;
                self.bin_int(def, inst, move |b, l, r| {
                    if s {
                        b.build_int_signed_rem(l, r, "sdiv").unwrap()
                    } else {
                        b.build_int_unsigned_rem(l, r, "udiv").unwrap()
                    }
                });
            }

            InstKind::FAdd => {
                self.bin_float(def, inst, |b, l, r| b.build_float_add(l, r, "add").unwrap())
            }
            InstKind::FSub => {
                self.bin_float(def, inst, |b, l, r| b.build_float_sub(l, r, "sub").unwrap())
            }
            InstKind::FMul => {
                self.bin_float(def, inst, |b, l, r| b.build_float_mul(l, r, "mul").unwrap())
            }
            InstKind::FDiv => {
                self.bin_float(def, inst, move |b, l, r| {
                    b.build_float_div(l, r, "udiv").unwrap()
                });
            }
            InstKind::FRem => {
                self.bin_float(def, inst, move |b, l, r| {
                    b.build_float_rem(l, r, "sdiv").unwrap()
                });
            }

            // ------------------------------------------------------------------
            // Bitwise / shifts
            // ------------------------------------------------------------------
            InstKind::And => self.bin_int(def, inst, |b, l, r| b.build_and(l, r, "and").unwrap()),
            InstKind::Or => self.bin_int(def, inst, |b, l, r| b.build_or(l, r, "or").unwrap()),
            InstKind::Xor => self.bin_int(def, inst, |b, l, r| b.build_xor(l, r, "xor").unwrap()),
            InstKind::LShl => self.bin_int(def, inst, |b, l, r| {
                b.build_left_shift(l, r, "shl").unwrap()
            }),
            InstKind::LShr => self.bin_int(def, inst, |b, l, r| {
                b.build_right_shift(l, r, false, "lshr").unwrap()
            }),
            InstKind::AShr => self.bin_int(def, inst, |b, l, r| {
                b.build_right_shift(l, r, true, "ashr").unwrap()
            }),

            // ------------------------------------------------------------------
            // Comparison
            // ------------------------------------------------------------------
            InstKind::Cmp(kind) => {
                let lhs = self.get_int_value(def, inst.operands[0]);
                let rhs = self.get_int_value(def, inst.operands[1]);
                let pred = self.map_int_pred(*kind);
                let val = self
                    .builder
                    .build_int_compare(pred, lhs, rhs, "cmp")
                    .unwrap();
                self.values.insert(inst.result.unwrap(), val.into());
            }

            // ------------------------------------------------------------------
            // Memory
            // ------------------------------------------------------------------
            InstKind::Alloca(ty) => {
                let llvm_ty = self.map_type(*ty);
                let ptr = self.builder.build_alloca(llvm_ty, "alloca").unwrap();
                self.values.insert(inst.result.unwrap(), ptr.into());
            }

            InstKind::NAlloca => {
                todo!("nalloca is not supported")
            }

            InstKind::Load { volatile } => {
                let ptr_val = self.get(inst.operands[0]).into_pointer_value();
                let result_ty = def.get_type(inst.result.unwrap());
                let llvm_ty = self.map_type(result_ty);
                let load = self.builder.build_load(llvm_ty, ptr_val, "load").unwrap();
                load.as_instruction_value()
                    .unwrap()
                    .set_volatile(*volatile)
                    .unwrap();
                self.values.insert(inst.result.unwrap(), load);
            }

            InstKind::Store { volatile } => {
                let ptr = self.get(inst.operands[0]).into_pointer_value();
                let value = self.get(inst.operands[1]);
                let store = self.builder.build_store(ptr, value).unwrap();
                store.set_volatile(*volatile).unwrap();
            }

            InstKind::ElementPtr => {
                let base = self.get(inst.operands[0]).into_pointer_value();
                let offset = self.get_int_value(def, inst.operands[1]);
                // GEP on i8* (opaque pointer model): byte-offset arithmetic.
                let i8_ty = self.context.i8_type();
                let ptr = unsafe {
                    self.builder
                        .build_gep(i8_ty, base, &[offset], "elemptr")
                        .unwrap()
                };
                self.values.insert(inst.result.unwrap(), ptr.into());
            }

            // ------------------------------------------------------------------
            // Function calls
            // ------------------------------------------------------------------
            InstKind::Call(fid) => {
                let callee = self.functions[fid];
                let args: Vec<BasicValueEnum> =
                    inst.operands.iter().map(|&op| self.get(op)).collect();

                let meta_args: Vec<_> = args.iter().map(|v| (*v).into()).collect();

                let call = self.builder.build_call(callee, &meta_args, "call").unwrap();

                if let Some(res) = inst.result {
                    let ret_val = call
                        .try_as_basic_value()
                        .basic()
                        .expect("call expected to return a value");
                    self.values.insert(res, ret_val);
                }
            }

            // ------------------------------------------------------------------
            // Casts
            // ------------------------------------------------------------------
            InstKind::Cast(kind) => {
                let src_val = self.get(inst.operands[0]);
                let dst_ty = def.get_type(inst.result.unwrap());
                let result = self.lower_cast(*kind, src_val, dst_ty);
                self.values.insert(inst.result.unwrap(), result);
            }

            // ------------------------------------------------------------------
            // Control flow
            // ------------------------------------------------------------------
            InstKind::Jump(target) => {
                let target_bb = self.blocks[target];
                self.builder.build_unconditional_branch(target_bb).unwrap();
            }

            InstKind::JumpIf {
                then_block,
                else_block,
            } => {
                let cond = self.get(inst.operands[0]).into_int_value();
                self.builder
                    .build_conditional_branch(
                        cond,
                        self.blocks[then_block],
                        self.blocks[else_block],
                    )
                    .unwrap();
            }

            InstKind::Ret => {
                if inst.operands.is_empty() {
                    self.builder.build_return(None).unwrap();
                } else {
                    let v = self.get(inst.operands[0]);
                    self.builder.build_return(Some(&v)).unwrap();
                }
            }
        }
    }

    // ---------------------------
    // HELPERS
    // ---------------------------

    fn lower_cast(
        &self,
        kind: CastKind,
        src: BasicValueEnum<'ctx>,
        dst_ty: Type,
    ) -> BasicValueEnum<'ctx> {
        match kind {
            CastKind::Zext => {
                let int_ty = self.map_int_type(dst_ty);
                self.builder
                    .build_int_z_extend(src.into_int_value(), int_ty, "zext")
                    .unwrap()
                    .into()
            }
            CastKind::Sext => {
                let int_ty = self.map_int_type(dst_ty);
                self.builder
                    .build_int_s_extend(src.into_int_value(), int_ty, "sext")
                    .unwrap()
                    .into()
            }
            CastKind::Trunc => {
                let int_ty = self.map_int_type(dst_ty);
                self.builder
                    .build_int_truncate(src.into_int_value(), int_ty, "trunc")
                    .unwrap()
                    .into()
            }
            CastKind::Bitcast => match (src, dst_ty) {
                (BasicValueEnum::IntValue(i), Type::F32) => self
                    .builder
                    .build_bit_cast(i, self.context.f32_type(), "bitcast")
                    .unwrap(),
                (BasicValueEnum::IntValue(i), Type::F64) => self
                    .builder
                    .build_bit_cast(i, self.context.f64_type(), "bitcast")
                    .unwrap(),
                (BasicValueEnum::FloatValue(f), Type::I32 | Type::I64 | Type::I8 | Type::I16) => {
                    let int_ty = self.map_int_type(dst_ty);
                    self.builder.build_bit_cast(f, int_ty, "bitcast").unwrap()
                }
                (v, _) => v,
            },
        }
    }

    fn bin_int<F>(&mut self, def: &FunctionDef, inst: &Inst, f: F)
    where
        F: Fn(
            &Builder<'ctx>,
            inkwell::values::IntValue<'ctx>,
            inkwell::values::IntValue<'ctx>,
        ) -> inkwell::values::IntValue<'ctx>,
    {
        let a = self.get_int_value(def, inst.operands[0]);
        let b = self.get_int_value(def, inst.operands[1]);
        let res = f(&self.builder, a, b);
        self.values.insert(inst.result.unwrap(), res.into());
    }

    fn bin_float<F>(&mut self, def: &FunctionDef, inst: &Inst, f: F)
    where
        F: Fn(
            &Builder<'ctx>,
            inkwell::values::FloatValue<'ctx>,
            inkwell::values::FloatValue<'ctx>,
        ) -> inkwell::values::FloatValue<'ctx>,
    {
        let a = self.get_float_value(def, inst.operands[0]);
        let b = self.get_float_value(def, inst.operands[1]);
        let res = f(&self.builder, a, b);
        self.values.insert(inst.result.unwrap(), res.into());
    }

    fn get_or_const(&self, def: &FunctionDef, v: ValueId) -> BasicValueEnum<'ctx> {
        if let Some(c) = def.try_get_iconst(v) {
            let ty = self.map_int_type(def.get_type(v));
            ty.const_int(c as u64, true).into()
        } else {
            self.get(v)
        }
    }

    fn get_int_value(&self, def: &FunctionDef, v: ValueId) -> IntValue<'ctx> {
        if let Some(c) = def.get_iconst(v) {
            let ty = self.map_int_type(def.get_type(v));
            ty.const_int(c as u64, true)
        } else {
            self.get(v).into_int_value()
        }
    }

    fn get_float_value(&self, def: &FunctionDef, v: ValueId) -> FloatValue<'ctx> {
        if let Some(c) = def.get_fconst(v) {
            let ty = self.map_float_type(def.get_type(v));
            ty.const_float(c)
        } else {
            self.get(v).into_float_value()
        }
    }

    fn get(&self, v: ValueId) -> BasicValueEnum<'ctx> {
        self.values.get(&v).copied().unwrap_or_else(|| {
            log::error!("missing value: {}", v);
            log::error!(
                "available values: {:?}",
                self.values.keys().collect::<Vec<_>>()
            );
            panic!("missing value: {}", v)
        })
    }
}
