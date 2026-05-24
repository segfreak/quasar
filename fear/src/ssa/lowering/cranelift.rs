use std::sync::Arc;

use cranelift::codegen::ir::{BlockArg, Function, LibCall, UserFuncName};
use cranelift::codegen::isa::{CallConv, TargetIsa};
use cranelift::prelude::*;
use cranelift_module::{Linkage as CLinkage, Module};
use cranelift_object::{ObjectBuilder, ObjectModule};

use crate::ssa::*;
use crate::types::{CallingConvention, FloatCmp, FunctionSignature, IntCmp, Linkage, Type};

use std::collections::HashMap;

fn map_type(ty: Type) -> types::Type {
    match ty {
        Type::I1 => types::I8,
        Type::I8 => types::I8,
        Type::I16 => types::I16,
        Type::I32 => types::I32,
        Type::I64 => types::I64,
        Type::F32 => types::F32,
        Type::F64 => types::F64,
        Type::Ptr => types::I64,
        Type::Void => panic!("void has no Cranelift type"),
    }
}

fn map_linkage(l: &Linkage) -> CLinkage {
    match l {
        Linkage::External => CLinkage::Export,
        Linkage::Internal => CLinkage::Local,
        Linkage::Weak => CLinkage::Preemptible,
    }
}

fn map_int_cond(kind: IntCmp) -> IntCC {
    match kind {
        IntCmp::Eq => IntCC::Equal,
        IntCmp::Ne => IntCC::NotEqual,
        IntCmp::Lt => IntCC::SignedLessThan,
        IntCmp::Le => IntCC::SignedLessThanOrEqual,
        IntCmp::Gt => IntCC::SignedGreaterThan,
        IntCmp::Ge => IntCC::SignedGreaterThanOrEqual,
        IntCmp::ULt => IntCC::UnsignedLessThan,
        IntCmp::ULe => IntCC::UnsignedLessThanOrEqual,
        IntCmp::UGt => IntCC::UnsignedGreaterThan,
        IntCmp::UGe => IntCC::UnsignedGreaterThanOrEqual,
    }
}

fn map_float_cond(kind: FloatCmp) -> FloatCC {
    match kind {
        FloatCmp::OEq => FloatCC::Equal,
        FloatCmp::ONe => FloatCC::NotEqual,
        FloatCmp::OLt => FloatCC::LessThan,
        FloatCmp::OLe => FloatCC::LessThanOrEqual,
        FloatCmp::OGt => FloatCC::GreaterThan,
        FloatCmp::OGe => FloatCC::GreaterThanOrEqual,

        FloatCmp::UEq => FloatCC::UnorderedOrEqual,
        // UNe: any unordered comparison is "not equal", use UnorderedOrLessThan | UnorderedOrGreaterThan
        // Cranelift does not have a single UnorderedOrNotEqual; use Ordered negation via Unordered.
        // The closest correct mapping: if either is NaN, they're "not equal".
        FloatCmp::UNe => FloatCC::UnorderedOrLessThan,
        FloatCmp::ULt => FloatCC::UnorderedOrLessThan,
        FloatCmp::ULe => FloatCC::UnorderedOrLessThanOrEqual,
        FloatCmp::UGt => FloatCC::UnorderedOrGreaterThan,
        FloatCmp::UGe => FloatCC::UnorderedOrGreaterThanOrEqual,

        FloatCmp::Ord => FloatCC::Ordered,
        FloatCmp::Uno => FloatCC::Unordered,
    }
}

pub struct CraneliftLowerer {
    pub module: ObjectModule,

    functions: HashMap<FuncId, cranelift_module::FuncId>,
}

impl CraneliftLowerer {
    pub fn new(
        name: &str,
        isa: Arc<dyn TargetIsa>,
        libcall_names: Box<dyn Fn(LibCall) -> String + Send + Sync>,
    ) -> Self {
        let builder = ObjectBuilder::new(isa, name, libcall_names).unwrap();
        let module = ObjectModule::new(builder);
        Self {
            module,
            functions: HashMap::new(),
        }
    }

    fn map_callconv(
        &self,
        target_conf: &isa::TargetFrontendConfig,
        callconv: CallingConvention,
    ) -> CallConv {
        use CallingConvention::*;

        match callconv {
            C =>
            /* C abi is a default abi on your machine */
            {
                target_conf.default_call_conv
            }
            SystemV => CallConv::SystemV,
            MicrosoftAbi => CallConv::WindowsFastcall,
        }
    }

    fn make_signature(&self, callconv: CallingConvention, sig: &FunctionSignature) -> Signature {
        let call_conv = self.map_callconv(&self.module.target_config(), callconv);
        let mut s = Signature::new(call_conv);
        for &p in &sig.params {
            s.params.push(AbiParam::new(map_type(p)));
        }
        if sig.returns != Type::Void {
            s.returns.push(AbiParam::new(map_type(sig.returns)));
        }
        s
    }

    pub fn lower_module(&mut self, m: &crate::ssa::Module) {
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

    pub fn declare_function(&mut self, m: &crate::ssa::Module, fid: FuncId) {
        let func = m.get_function(fid).unwrap();
        let sig = self.make_signature(func.calling_convention, &func.signature);
        let cl_fid = self
            .module
            .declare_function(&func.name, map_linkage(&func.linkage), &sig)
            .unwrap();
        self.functions.insert(fid, cl_fid);
    }

    pub fn lower_function(&mut self, m: &crate::ssa::Module, fid: FuncId) {
        let ir_func = m.get_function(fid).unwrap();
        let def = ir_func.get_definition().unwrap();
        let cl_fid = self.functions[&fid];

        let sig = self.make_signature(ir_func.calling_convention, &ir_func.signature);
        let mut cl_func =
            Function::with_name_signature(UserFuncName::user(0, cl_fid.as_u32()), sig);

        let mut func_refs: HashMap<cranelift_module::FuncId, cranelift::codegen::ir::FuncRef> =
            HashMap::new();
        for &cl_callee_fid in self.functions.values() {
            let fref = self
                .module
                .declare_func_in_func(cl_callee_fid, &mut cl_func);
            func_refs.insert(cl_callee_fid, fref);
        }

        let mut fctx = FunctionBuilderContext::new();
        let mut fx = FunctionBuilder::new(&mut cl_func, &mut fctx);

        let mut values: HashMap<ValueId, cranelift::prelude::Value> = HashMap::new();
        let mut blocks: HashMap<BlockId, cranelift::prelude::Block> = HashMap::new();

        let entry = def.entry;
        for &bid in def.blocks.keys() {
            let cl_block = fx.create_block();
            blocks.insert(bid, cl_block);
        }

        {
            let entry_block = blocks[&entry];
            fx.append_block_params_for_function_params(entry_block);
            fx.switch_to_block(entry_block);
            fx.seal_block(entry_block);

            let entry_ir = &def.blocks[&entry];
            for (i, &param_val) in entry_ir.params.iter().enumerate() {
                let cl_val = fx.block_params(entry_block)[i];
                values.insert(param_val, cl_val);
            }
        }

        for (&bid, block) in &def.blocks {
            if bid == entry {
                continue;
            }
            let cl_block = blocks[&bid];
            for &param in &block.params {
                let ty = def.get_type(param);
                let cl_val = fx.append_block_param(cl_block, map_type(ty));
                values.insert(param, cl_val);
            }
        }

        let order: Vec<BlockId> = def.reverse_post_order();

        for &bid in &order {
            if bid != entry {
                fx.switch_to_block(blocks[&bid]);
            }

            let block = &def.blocks[&bid];
            for &inst_id in &block.insts {
                let inst = &def.insts[&inst_id];
                compile_inst(
                    def,
                    inst,
                    &mut fx,
                    &mut values,
                    &blocks,
                    &self.functions,
                    &func_refs,
                );
            }
        }

        for &bid in def.blocks.keys() {
            if bid != entry {
                fx.seal_block(blocks[&bid]);
            }
        }

        fx.finalize();

        let mut ctx = cranelift::codegen::Context::new();
        ctx.func = cl_func;
        self.module.define_function(cl_fid, &mut ctx).unwrap();
    }

    pub fn finish(self) -> Vec<u8> {
        self.module.finish().emit().unwrap()
    }

    pub fn get_module(&self) -> &ObjectModule {
        &self.module
    }
}

fn get_or_const(
    def: &FunctionDef,
    v: ValueId,
    values: &HashMap<ValueId, cranelift::prelude::Value>,
    fx: &mut FunctionBuilder,
) -> cranelift::prelude::Value {
    if let Some(c) = def.get_iconst(v) {
        let ty = map_type(def.get_type(v));
        fx.ins().iconst(ty, c)
    } else {
        *values
            .get(&v)
            .unwrap_or_else(|| panic!("missing value: {}", v))
    }
}

fn get_float_or_const(
    def: &FunctionDef,
    v: ValueId,
    values: &HashMap<ValueId, cranelift::prelude::Value>,
    fx: &mut FunctionBuilder,
) -> cranelift::prelude::Value {
    if let Some(f) = def.get_fconst(v) {
        match def.get_type(v) {
            Type::F32 => fx.ins().f32const(f as f32),
            Type::F64 => fx.ins().f64const(f),
            _ => panic!("fconst on non-float type"),
        }
    } else {
        *values
            .get(&v)
            .unwrap_or_else(|| panic!("missing value: {}", v))
    }
}

fn get(
    v: ValueId,
    values: &HashMap<ValueId, cranelift::prelude::Value>,
) -> cranelift::prelude::Value {
    *values
        .get(&v)
        .unwrap_or_else(|| panic!("missing value: {}", v))
}

fn compile_inst(
    def: &FunctionDef,
    inst: &Inst,
    fx: &mut FunctionBuilder,
    values: &mut HashMap<ValueId, cranelift::prelude::Value>,
    blocks: &HashMap<BlockId, cranelift::prelude::Block>,
    functions: &HashMap<FuncId, cranelift_module::FuncId>,
    func_refs: &HashMap<cranelift_module::FuncId, cranelift::codegen::ir::FuncRef>,
) {
    log::trace!(
        "lowering inst: {:?}, operands: {:?}",
        inst.kind,
        inst.operands
    );

    match &inst.kind {
        InstKind::IConst(x) => {
            let ty = map_type(def.get_type(inst.result.unwrap()));
            let val = fx.ins().iconst(ty, *x);
            values.insert(inst.result.unwrap(), val);
        }

        InstKind::FConst(bits) => {
            let ty = def.get_type(inst.result.unwrap());
            let val = match ty {
                Type::F32 => fx.ins().f32const(f32::from_bits(*bits as u32)),
                Type::F64 => fx.ins().f64const(f64::from_bits(*bits)),
                _ => panic!("FConst with non-float type"),
            };
            values.insert(inst.result.unwrap(), val);
        }

        InstKind::Add => bin_int(def, inst, values, fx, |fx, a, b| fx.ins().iadd(a, b)),
        InstKind::Sub => bin_int(def, inst, values, fx, |fx, a, b| fx.ins().isub(a, b)),
        InstKind::Mul => bin_int(def, inst, values, fx, |fx, a, b| fx.ins().imul(a, b)),
        InstKind::Div { signed: true } => {
            bin_int(def, inst, values, fx, |fx, a, b| fx.ins().sdiv(a, b))
        }
        InstKind::Div { signed: false } => {
            bin_int(def, inst, values, fx, |fx, a, b| fx.ins().udiv(a, b))
        }
        InstKind::Rem { signed: true } => {
            bin_int(def, inst, values, fx, |fx, a, b| fx.ins().srem(a, b))
        }
        InstKind::Rem { signed: false } => {
            bin_int(def, inst, values, fx, |fx, a, b| fx.ins().urem(a, b))
        }

        InstKind::FAdd => bin_float(def, inst, values, fx, |fx, a, b| fx.ins().fadd(a, b)),
        InstKind::FSub => bin_float(def, inst, values, fx, |fx, a, b| fx.ins().fsub(a, b)),
        InstKind::FMul => bin_float(def, inst, values, fx, |fx, a, b| fx.ins().fmul(a, b)),
        InstKind::FDiv => bin_float(def, inst, values, fx, |fx, a, b| fx.ins().fdiv(a, b)),
        InstKind::FRem => {
            panic!("frem is not a native Cranelift opcode; use an fmodf/fmod libcall")
        }

        InstKind::Not => un_int(def, inst, values, fx, |fx, v| fx.ins().bnot(v)),
        InstKind::And => bin_int(def, inst, values, fx, |fx, a, b| fx.ins().band(a, b)),
        InstKind::Or => bin_int(def, inst, values, fx, |fx, a, b| fx.ins().bor(a, b)),
        InstKind::Xor => bin_int(def, inst, values, fx, |fx, a, b| fx.ins().bxor(a, b)),
        InstKind::LShl => bin_int(def, inst, values, fx, |fx, a, b| fx.ins().ishl(a, b)),
        InstKind::LShr => bin_int(def, inst, values, fx, |fx, a, b| fx.ins().ushr(a, b)),
        InstKind::AShr => bin_int(def, inst, values, fx, |fx, a, b| fx.ins().sshr(a, b)),

        InstKind::Cmp(kind) => {
            let lhs = get_or_const(def, inst.operands[0], values, fx);
            let rhs = get_or_const(def, inst.operands[1], values, fx);
            let cond = map_int_cond(*kind);
            let val = fx.ins().icmp(cond, lhs, rhs);
            values.insert(inst.result.unwrap(), val);
        }

        InstKind::FCmp(kind) => {
            let lhs = get_float_or_const(def, inst.operands[0], values, fx);
            let rhs = get_float_or_const(def, inst.operands[1], values, fx);
            let cond = map_float_cond(*kind);
            let val = fx.ins().fcmp(cond, lhs, rhs);
            values.insert(inst.result.unwrap(), val);
        }

        InstKind::Alloca(ty) => {
            let ty = map_type(*ty);
            let size = ty.bytes();
            let slot = fx.create_sized_stack_slot(StackSlotData::new(
                StackSlotKind::ExplicitSlot,
                size,
                0,
            ));
            let ptr = fx.ins().stack_addr(types::I64, slot, 0);
            values.insert(inst.result.unwrap(), ptr);
        }

        InstKind::NAlloca(ty, count) => {
            let ty = map_type(*ty);
            let elem_bytes = ty.bytes();
            let size = elem_bytes * (*count as u32);
            let slot_data = StackSlotData::new(StackSlotKind::ExplicitSlot, size, 0);
            let slot = fx.create_sized_stack_slot(slot_data);
            let ptr = fx.ins().stack_addr(types::I64, slot, 0);
            values.insert(inst.result.unwrap(), ptr);
        }

        InstKind::Load { volatile } => {
            let ptr = get(inst.operands[0], values);
            let result_ty = map_type(def.get_type(inst.result.unwrap()));
            let mut flags = MemFlags::new();
            if !volatile {
                flags.set_notrap();
            }
            let val = fx.ins().load(result_ty, flags, ptr, 0);
            values.insert(inst.result.unwrap(), val);
        }

        InstKind::Store { volatile } => {
            let ptr = get(inst.operands[0], values);
            let value = get(inst.operands[1], values);
            let mut flags = MemFlags::new();
            if !volatile {
                flags.set_notrap();
            }
            fx.ins().store(flags, value, ptr, 0);
        }

        InstKind::PtrOffset => {
            let base = get(inst.operands[0], values);
            let offset = get_or_const(def, inst.operands[1], values, fx);
            let ptr = fx.ins().iadd(base, offset);
            values.insert(inst.result.unwrap(), ptr);
        }

        InstKind::ElementPtr(ty) => {
            let base = get(inst.operands[0], values);
            let base_ty = fx.func.dfg.value_type(base);

            let raw_offset = get_or_const(def, inst.operands[1], values, fx);
            let offset_ty = fx.func.dfg.value_type(raw_offset);

            let offset_expanded = if offset_ty.bits() < 64 {
                fx.ins().sextend(types::I64, raw_offset)
            } else {
                raw_offset
            };

            let type_size = fx.ins().iconst(types::I64, ty.get_size() as i64);
            let final_offset = fx.ins().imul(offset_expanded, type_size);

            let final_offset_adjusted = if base_ty != types::I64 {
                fx.ins().ireduce(base_ty, final_offset)
            } else {
                final_offset
            };

            let ptr = fx.ins().iadd(base, final_offset_adjusted);
            values.insert(inst.result.unwrap(), ptr);
        }

        InstKind::Call(fid) => {
            let cl_fid = functions
                .get(fid)
                .unwrap_or_else(|| panic!("unknown FuncId {:?} in Call", fid));
            let func_ref = func_refs
                .get(cl_fid)
                .unwrap_or_else(|| panic!("no FuncRef for cl_fid {:?}", cl_fid));

            let args: Vec<cranelift::prelude::Value> = inst
                .operands
                .iter()
                .map(|&v| get_or_const(def, v, values, fx))
                .collect();

            let call_inst = fx.ins().call(*func_ref, &args);

            if let Some(result) = inst.result {
                let results = fx.inst_results(call_inst);
                assert!(
                    !results.is_empty(),
                    "Call has result ValueId but callee returns nothing"
                );
                values.insert(result, results[0]);
            }
        }

        InstKind::Cast(kind) => {
            let src = get(inst.operands[0], values);
            let src_ty = def.get_type(inst.operands[0]);
            let dst_ty = def.get_type(inst.result.unwrap());
            let result = lower_cast(*kind, src, src_ty, dst_ty, fx);
            values.insert(inst.result.unwrap(), result);
        }

        InstKind::Jump(target) => {
            let target_bb = blocks[target];

            let args: Vec<BlockArg> = inst
                .operands
                .iter()
                .map(|&op| BlockArg::Value(get_or_const(def, op, values, fx)))
                .collect();

            fx.ins().jump(target_bb, &args);
        }

        InstKind::JumpIf {
            then_block,
            else_block,
        } => {
            let cond = get(inst.operands[0], values);

            let (_, then_params, else_params) = def.get_jumpif_params(inst).unwrap();

            let then_bb = blocks[then_block];
            let else_bb = blocks[else_block];

            let then_args: Vec<BlockArg> = then_params
                .iter()
                .map(|&op| BlockArg::Value(get_or_const(def, op, values, fx)))
                .collect();
            let else_args: Vec<BlockArg> = else_params
                .iter()
                .map(|&op| BlockArg::Value(get_or_const(def, op, values, fx)))
                .collect();

            fx.ins()
                .brif(cond, then_bb, &then_args, else_bb, &else_args);
        }

        InstKind::Ret => {
            if inst.operands.is_empty() {
                fx.ins().return_(&[]);
            } else {
                let v = get(inst.operands[0], values);
                fx.ins().return_(&[v]);
            }
        }
    }
}

fn lower_cast(
    kind: CastKind,
    src: cranelift::prelude::Value,
    src_ty: Type,
    dst_ty: Type,
    fx: &mut FunctionBuilder,
) -> cranelift::prelude::Value {
    let cl_dst = map_type(dst_ty);

    match kind {
        CastKind::Zext => fx.ins().uextend(cl_dst, src),
        CastKind::Sext => fx.ins().sextend(cl_dst, src),
        CastKind::Trunc => fx.ins().ireduce(cl_dst, src),
        CastKind::Bitcast => match (src_ty, dst_ty) {
            (Type::I32, Type::F32)
            | (Type::I64, Type::F64)
            | (Type::F32, Type::I32)
            | (Type::F64, Type::I64) => fx.ins().bitcast(cl_dst, MemFlags::new(), src),
            _ => src,
        },
        CastKind::SIToFP => fx.ins().fcvt_from_sint(cl_dst, src),
        CastKind::UIToFP => fx.ins().fcvt_from_uint(cl_dst, src),
        CastKind::FPToSI => fx.ins().fcvt_to_sint(cl_dst, src),
        CastKind::FPToUI => fx.ins().fcvt_to_uint(cl_dst, src),
        CastKind::FPromote => fx.ins().fpromote(cl_dst, src),
        CastKind::FTrunc => fx.ins().fdemote(cl_dst, src),
    }
}

fn bin_int<F>(
    def: &FunctionDef,
    inst: &Inst,
    values: &mut HashMap<ValueId, cranelift::prelude::Value>,
    fx: &mut FunctionBuilder,
    f: F,
) where
    F: Fn(
        &mut FunctionBuilder,
        cranelift::prelude::Value,
        cranelift::prelude::Value,
    ) -> cranelift::prelude::Value,
{
    let a = get_or_const(def, inst.operands[0], values, fx);
    let b = get_or_const(def, inst.operands[1], values, fx);
    let res = f(fx, a, b);
    values.insert(inst.result.unwrap(), res);
}

fn un_int<F>(
    def: &FunctionDef,
    inst: &Inst,
    values: &mut HashMap<ValueId, cranelift::prelude::Value>,
    fx: &mut FunctionBuilder,
    f: F,
) where
    F: Fn(&mut FunctionBuilder, cranelift::prelude::Value) -> cranelift::prelude::Value,
{
    let a = get_or_const(def, inst.operands[0], values, fx);
    let res = f(fx, a);
    values.insert(inst.result.unwrap(), res);
}

fn bin_float<F>(
    def: &FunctionDef,
    inst: &Inst,
    values: &mut HashMap<ValueId, cranelift::prelude::Value>,
    fx: &mut FunctionBuilder,
    f: F,
) where
    F: Fn(
        &mut FunctionBuilder,
        cranelift::prelude::Value,
        cranelift::prelude::Value,
    ) -> cranelift::prelude::Value,
{
    let a = get_float_or_const(def, inst.operands[0], values, fx);
    let b = get_float_or_const(def, inst.operands[1], values, fx);
    let res = f(fx, a, b);
    values.insert(inst.result.unwrap(), res);
}
