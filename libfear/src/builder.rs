use fear::types::{CastKind, FloatCmp, IntCmp};

use crate::{types::*, *};

/// Creates a new basic block inside the function to build branches or loops.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn fearCreateBlock(f: *mut FearFunctionDef) -> FearBlockId {
    as_def(f).new_block()
}

/// Adds a top-level input parameter to the function declaration.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn fearCreateFuncParam(f: *mut FearFunctionDef, ty: FearType) -> FearBlockId {
    as_def(f).add_param(ty.into())
}

/// Appends a parameter to a specific basic block, used to pass values in SSA form (instead of PHI nodes).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn fearCreateBlockParam(
    f: *mut FearFunctionDef,
    block: FearBlockId,
    ty: FearType,
) -> FearBlockId {
    as_def(f).add_block_param(block, ty.into())
}

/// Generates an integer constant value inside a target basic block.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn fearCreateIntConst(
    f: *mut FearFunctionDef,
    parent: FearBlockId,
    ty: FearType,
    val: i64,
) -> FearValueId {
    as_def(f).make_iconst(parent, ty.into(), val)
}

/// Emits an instruction to allocate stack space for a local variable of the specified type.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn fearCreateAlloca(
    f: *mut FearFunctionDef,
    parent: FearBlockId,
    ty: FearType,
) -> FearValueId {
    as_def(f).make_alloca(parent, ty.into())
}

/// Emits an instruction to allocate stack space for a local variable of the specified type.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn fearCreateArrayAlloca(
    f: *mut FearFunctionDef,
    parent: FearBlockId,
    ty: FearType,
    count: u32,
) -> FearValueId {
    as_def(f).make_nalloca(parent, ty.into(), count as usize)
}

/// Emits an instruction to get a pointer with offset (byte-addressed).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn fearCreatePtrOffset(
    f: *mut FearFunctionDef,
    parent: FearBlockId,
    base: FearValueId,
    offset: FearValueId,
) -> FearValueId {
    as_def(f).make_ptr_offset(parent, base, offset)
}

/// Emits an instruction to get a pointer with offset (element-addressed).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn fearCreateElementPtr(
    f: *mut FearFunctionDef,
    parent: FearBlockId,
    ty: FearType,
    base: FearValueId,
    offset: FearValueId,
) -> FearValueId {
    as_def(f).make_element_ptr(parent, Type::from(ty), base, offset)
}

/// Emits an instruction to load a value of type `ty` from a memory address pointed to by `ptr`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn fearCreateLoad(
    f: *mut FearFunctionDef,
    parent: FearBlockId,
    ty: FearType,
    ptr: FearValueId,
) -> FearValueId {
    as_def(f).make_load(parent, false, ty.into(), ptr)
}

/// Emits an instruction to store a `value` into a memory address specified by `ptr`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn fearCreateStore(
    f: *mut FearFunctionDef,
    parent: FearBlockId,
    ptr: FearValueId,
    value: FearValueId,
) {
    as_def(f).make_store(parent, false, ptr, value);
}

/// Emits a volatile load instruction, ensuring memory access is not optimized away or reordered.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn fearCreateVolatileLoad(
    f: *mut FearFunctionDef,
    parent: FearBlockId,
    ty: FearType,
    ptr: FearValueId,
) -> FearValueId {
    as_def(f).make_load(parent, true, ty.into(), ptr)
}

/// Emits a volatile store instruction, ensuring the memory write occurs exactly as requested.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn fearCreateVolatileStore(
    f: *mut FearFunctionDef,
    parent: FearBlockId,
    ptr: FearValueId,
    value: FearValueId,
) {
    as_def(f).make_store(parent, true, ptr, value);
}

/// Appends a function call instruction to a basic block with an array of argument identifiers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn fearCreateCall(
    f: *mut FearFunctionDef,
    parent: FearBlockId,
    func: FearFuncId,
    ty: FearType,
    args: *const FearValueId,
    nargs: u32,
) -> FearValueId {
    as_def(f).make_call(parent, ty.into(), func, to_vec(args, nargs as usize))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn fearCreateAdd(
    f: *mut FearFunctionDef,
    parent: FearBlockId,
    ty: FearType,
    a: FearValueId,
    b: FearValueId,
) -> FearValueId {
    as_def(f).make_add(parent, ty.into(), a, b)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn fearCreateSub(
    f: *mut FearFunctionDef,
    parent: FearBlockId,
    ty: FearType,
    a: FearValueId,
    b: FearValueId,
) -> FearValueId {
    as_def(f).make_sub(parent, ty.into(), a, b)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn fearCreateMul(
    f: *mut FearFunctionDef,
    parent: FearBlockId,
    ty: FearType,
    a: FearValueId,
    b: FearValueId,
) -> FearValueId {
    as_def(f).make_mul(parent, ty.into(), a, b)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn fearCreateDiv(
    f: *mut FearFunctionDef,
    parent: FearBlockId,
    ty: FearType,
    a: FearValueId,
    b: FearValueId,
) -> FearValueId {
    as_def(f).make_div(parent, true, ty.into(), a, b)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn fearCreateUnsignedDiv(
    f: *mut FearFunctionDef,
    parent: FearBlockId,
    ty: FearType,
    a: FearValueId,
    b: FearValueId,
) -> FearValueId {
    as_def(f).make_div(parent, false, ty.into(), a, b)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn fearCreateRem(
    f: *mut FearFunctionDef,
    parent: FearBlockId,
    ty: FearType,
    a: FearValueId,
    b: FearValueId,
) -> FearValueId {
    as_def(f).make_rem(parent, true, ty.into(), a, b)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn fearCreateUnsignedRem(
    f: *mut FearFunctionDef,
    parent: FearBlockId,
    ty: FearType,
    a: FearValueId,
    b: FearValueId,
) -> FearValueId {
    as_def(f).make_rem(parent, false, ty.into(), a, b)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn fearCreateFloatAdd(
    f: *mut FearFunctionDef,
    parent: FearBlockId,
    ty: FearType,
    a: FearValueId,
    b: FearValueId,
) -> FearValueId {
    as_def(f).make_fadd(parent, ty.into(), a, b)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn fearCreateFloatSub(
    f: *mut FearFunctionDef,
    parent: FearBlockId,
    ty: FearType,
    a: FearValueId,
    b: FearValueId,
) -> FearValueId {
    as_def(f).make_fsub(parent, ty.into(), a, b)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn fearCreateFloatMul(
    f: *mut FearFunctionDef,
    parent: FearBlockId,
    ty: FearType,
    a: FearValueId,
    b: FearValueId,
) -> FearValueId {
    as_def(f).make_fmul(parent, ty.into(), a, b)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn fearCreateFloatDiv(
    f: *mut FearFunctionDef,
    parent: FearBlockId,
    ty: FearType,
    a: FearValueId,
    b: FearValueId,
) -> FearValueId {
    as_def(f).make_fdiv(parent, ty.into(), a, b)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn fearCreateFloatRem(
    f: *mut FearFunctionDef,
    parent: FearBlockId,
    ty: FearType,
    a: FearValueId,
    b: FearValueId,
) -> FearValueId {
    as_def(f).make_frem(parent, ty.into(), a, b)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn fearCreateBitwiseNot(
    f: *mut FearFunctionDef,
    parent: FearBlockId,
    ty: FearType,
    v: FearValueId,
) -> FearValueId {
    as_def(f).make_not(parent, ty.into(), v)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn fearCreateBitwiseAnd(
    f: *mut FearFunctionDef,
    parent: FearBlockId,
    ty: FearType,
    a: FearValueId,
    b: FearValueId,
) -> FearValueId {
    as_def(f).make_and(parent, ty.into(), a, b)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn fearCreateBitwiseOr(
    f: *mut FearFunctionDef,
    parent: FearBlockId,
    ty: FearType,
    a: FearValueId,
    b: FearValueId,
) -> FearValueId {
    as_def(f).make_or(parent, ty.into(), a, b)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn fearCreateBitwiseXor(
    f: *mut FearFunctionDef,
    parent: FearBlockId,
    ty: FearType,
    a: FearValueId,
    b: FearValueId,
) -> FearValueId {
    as_def(f).make_xor(parent, ty.into(), a, b)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn fearCreateLogicalShiftLeft(
    f: *mut FearFunctionDef,
    parent: FearBlockId,
    ty: FearType,
    a: FearValueId,
    b: FearValueId,
) -> FearValueId {
    as_def(f).make_lshl(parent, ty.into(), a, b)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn fearCreateLogicalShiftRight(
    f: *mut FearFunctionDef,
    parent: FearBlockId,
    ty: FearType,
    a: FearValueId,
    b: FearValueId,
) -> FearValueId {
    as_def(f).make_lshr(parent, ty.into(), a, b)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn fearCreateArithShiftRight(
    f: *mut FearFunctionDef,
    parent: FearBlockId,
    ty: FearType,
    a: FearValueId,
    b: FearValueId,
) -> FearValueId {
    as_def(f).make_ashr(parent, ty.into(), a, b)
}

/// Terminates a block with an unconditional jump to another basic block, passing arguments to its block parameters.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn fearCreateJump(
    f: *mut FearFunctionDef,
    parent: FearBlockId,
    target: FearBlockId,
    params: *const FearValueId,
    nparams: u32,
) {
    as_def(f).make_jump(parent, target, to_vec(params, nparams as usize))
}

/// Compares two integers
#[unsafe(no_mangle)]
pub unsafe extern "C" fn fearCreateIntCompare(
    f: *mut FearFunctionDef,
    parent: FearBlockId,
    pred: FearIntCmp,
    lhs: FearValueId,
    rhs: FearValueId,
) -> FearValueId {
    as_def(f).make_cmp(parent, IntCmp::from(pred), lhs, rhs)
}

/// Compares two floatpoint values
#[unsafe(no_mangle)]
pub unsafe extern "C" fn fearCreateFloatCompare(
    f: *mut FearFunctionDef,
    parent: FearBlockId,
    pred: FearFloatCmp,
    lhs: FearValueId,
    rhs: FearValueId,
) -> FearValueId {
    as_def(f).make_fcmp(parent, FloatCmp::from(pred), lhs, rhs)
}

/// Emits an instruction to zero-extend an integer to a wider integer type.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn fearCreateZeroExt(
    f: *mut FearFunctionDef,
    parent: FearBlockId,
    ty: FearType,
    src: FearValueId,
) -> FearValueId {
    as_def(f).make_cast(parent, CastKind::Zext, ty.into(), src)
}

/// Emits an instruction to sign-extend an integer to a wider integer type.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn fearCreateSignExt(
    f: *mut FearFunctionDef,
    parent: FearBlockId,
    ty: FearType,
    src: FearValueId,
) -> FearValueId {
    as_def(f).make_cast(parent, CastKind::Sext, ty.into(), src)
}

/// Emits an instruction to truncate an integer to a narrower integer type.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn fearCreateTrunc(
    f: *mut FearFunctionDef,
    parent: FearBlockId,
    ty: FearType,
    src: FearValueId,
) -> FearValueId {
    as_def(f).make_cast(parent, CastKind::Trunc, ty.into(), src)
}

/// Emits an instruction to bitcast a value to another type of the same bit width without changing the raw bits.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn fearCreateBitcast(
    f: *mut FearFunctionDef,
    parent: FearBlockId,
    ty: FearType,
    src: FearValueId,
) -> FearValueId {
    as_def(f).make_cast(parent, CastKind::Bitcast, ty.into(), src)
}

/// Emits an instruction to convert a signed integer to a floating-point value.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn fearCreateIntToFloat(
    f: *mut FearFunctionDef,
    parent: FearBlockId,
    ty: FearType,
    src: FearValueId,
) -> FearValueId {
    as_def(f).make_cast(parent, CastKind::SIToFP, ty.into(), src)
}

/// Emits an instruction to convert an unsigned integer to a floating-point value.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn fearCreateUnsignedIntToFloat(
    f: *mut FearFunctionDef,
    parent: FearBlockId,
    ty: FearType,
    src: FearValueId,
) -> FearValueId {
    as_def(f).make_cast(parent, CastKind::UIToFP, ty.into(), src)
}

/// Emits an instruction to convert a floating-point value to a signed integer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn fearCreateFloatToInt(
    f: *mut FearFunctionDef,
    parent: FearBlockId,
    ty: FearType,
    src: FearValueId,
) -> FearValueId {
    as_def(f).make_cast(parent, CastKind::FPToSI, ty.into(), src)
}

/// Emits an instruction to convert a floating-point value to an unsigned integer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn fearCreateFloatToUnsignedInt(
    f: *mut FearFunctionDef,
    parent: FearBlockId,
    ty: FearType,
    src: FearValueId,
) -> FearValueId {
    as_def(f).make_cast(parent, CastKind::FPToUI, ty.into(), src)
}

/// Emits an instruction to extend a floating-point value to a higher precision floating-point type (e.g., f32 to f64).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn fearCreateFloatPromote(
    f: *mut FearFunctionDef,
    parent: FearBlockId,
    ty: FearType,
    src: FearValueId,
) -> FearValueId {
    as_def(f).make_cast(parent, CastKind::FPromote, ty.into(), src)
}

/// Emits an instruction to truncate a floating-point value to a lower precision floating-point type (e.g., f64 to f32).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn fearCreateFloatTrunc(
    f: *mut FearFunctionDef,
    parent: FearBlockId,
    ty: FearType,
    src: FearValueId,
) -> FearValueId {
    as_def(f).make_cast(parent, CastKind::FTrunc, ty.into(), src)
}

/// Terminates a block with a conditional jump. Switches control flow to block `t` (true) or `e` (false) depending on `cond`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn fearCreateCondJump(
    f: *mut FearFunctionDef,
    parent: FearBlockId,
    cond: FearValueId,
    t: FearBlockId,
    targs: *const FearValueId,
    tn: u32,
    e: FearBlockId,
    eargs: *const FearValueId,
    en: u32,
) {
    as_def(f).make_jumpif(
        parent,
        cond,
        t,
        to_vec(targs, tn as usize),
        e,
        to_vec(eargs, en as usize),
    )
}

/// Terminates a block with a return instruction passing a return value.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn fearCreateRet(
    f: *mut FearFunctionDef,
    parent: FearBlockId,
    v: FearValueId,
) {
    as_def(f).make_ret(parent, Some(v))
}

/// Terminates a block with a void return instruction.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn fearCreateRetVoid(f: *mut FearFunctionDef, parent: FearBlockId) {
    as_def(f).make_ret(parent, None)
}
