#![allow(clippy::missing_safety_doc, unsafe_op_in_unsafe_fn)]

use std::ffi::c_int;
use std::io::Write;
use std::os::fd::FromRawFd;
use std::os::raw::c_char;
use std::{ffi::CStr, ptr};

use fear::compiler::{self, Backend, CompilerConfig, OutputType};
use fear::types::OptLevel;
use fear::{binary, ir::*};
use fearcore::{target::*, *};

use target_lexicon::Triple;

/* ========================= OPAQUE FFI TYPES ========================= */

/// opaque
#[repr(C)]
pub struct FearModule {
    __: [i8; 0],
}

/// opaque
#[repr(C)]
pub struct FearFunctionDef {
    __: [i8; 0],
}

pub type FearFuncId = u32;
pub type FearBlockId = u32;
pub type FearValueId = u32;

#[repr(C)]
#[derive(Clone, Copy)]
pub enum FearType {
    FearVoid,
    FearBool,
    FearInt8,
    FearInt16,
    FearInt32,
    FearInt64,
    FearFloat32,
    FearFloat64,
    FearPointer,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub enum FearOptLevel {
    FearOptNone,
    FearOptDefault,
    FearOptFull,
}

impl From<FearType> for Type {
    fn from(t: FearType) -> Self {
        match t {
            FearType::FearVoid => Type::Void,
            FearType::FearBool => Type::I1,
            FearType::FearInt8 => Type::I8,
            FearType::FearInt16 => Type::I16,
            FearType::FearInt32 => Type::I32,
            FearType::FearInt64 => Type::I64,
            FearType::FearFloat32 => Type::F32,
            FearType::FearFloat64 => Type::F64,
            FearType::FearPointer => Type::Ptr,
        }
    }
}

impl From<FearOptLevel> for OptLevel {
    fn from(t: FearOptLevel) -> Self {
        match t {
            FearOptLevel::FearOptNone => OptLevel::None,
            FearOptLevel::FearOptDefault => OptLevel::Default,
            FearOptLevel::FearOptFull => OptLevel::Full,
        }
    }
}

/* ========================= INTERNAL HELPERS ========================= */

fn cstr(s: *const c_char) -> String {
    unsafe { CStr::from_ptr(s).to_string_lossy().into_owned() }
}

unsafe fn to_vec<T: Clone>(data: *const T, nelem: usize) -> Vec<T> {
    if data.is_null() || nelem == 0 {
        return Vec::new();
    }
    std::slice::from_raw_parts(data, nelem).to_vec()
}

unsafe fn as_module(m: *mut FearModule) -> &'static mut Module {
    &mut *(m as *mut Module)
}

unsafe fn as_def(f: *mut FearFunctionDef) -> &'static mut FunctionDef {
    &mut *(f as *mut FunctionDef)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn fearLoweringHasLLVM() -> bool {
    fear::lowering::has_llvm()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn fearLoweringHasCranelift() -> bool {
    fear::lowering::has_cranelift()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn fearEmitCraneliftObjectToFile(
    m: *mut FearModule,
    opt: FearOptLevel,
    fd: c_int,
) {
    let m = as_module(m);
    let config = CompilerConfig {
        backend: Backend::Cranelift,
        output_type: OutputType::Object,
        triple: Triple::host(),
        opt_level: OptLevel::from(opt),
    };
    let file = unsafe { std::fs::File::from_raw_fd(fd) };
    compiler::compile_module(m, &config, file).unwrap();
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn fearModuleCreate(name: *const c_char) -> *mut FearModule {
    let m = Module::new(cstr(name));
    Box::into_raw(Box::new(m)) as *mut FearModule
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn fearBinaryFetchModuleFromFile(fd: c_int) -> *mut FearModule {
    let file = unsafe { std::fs::File::from_raw_fd(fd) };
    let m = fear::binary::read::<fear::ir::Module, _>(file).unwrap();
    Box::into_raw(Box::new(m)) as *mut FearModule
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn fearModuleDispose(m: *mut FearModule) {
    if !m.is_null() {
        drop(Box::from_raw(m));
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn fearModuleOptimize(m: *mut FearModule) {
    let m = as_module(m);
    m.optimize();
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn fearDumpToStringBuffer(m: *mut FearModule, buf: *mut c_char, len: u32) {
    let m = as_module(m);
    let s = m.dump();
    if buf.is_null() || len == 0 {
        return;
    }
    let bytes = s.as_bytes();
    let max_copy = (len as usize).saturating_sub(1);
    let copy_len = bytes.len().min(max_copy);
    ptr::copy_nonoverlapping(bytes.as_ptr(), buf as *mut u8, copy_len);
    *buf.add(copy_len) = 0;
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn fearDumpToFile(m: *mut FearModule, fd: c_int) {
    let m = as_module(m);
    let s = m.dump();
    let mut file = unsafe { std::fs::File::from_raw_fd(fd) };
    let _ = file.write_all(s.as_bytes());
    std::mem::forget(file);
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn fearBinaryDumpToFile(m: *mut FearModule, fd: c_int) {
    let m = as_module(m);
    let file = unsafe { std::fs::File::from_raw_fd(fd) };
    binary::write(m, file).unwrap();
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn fearDeclareFunction(
    m: *mut FearModule,
    name: *const c_char,
    params: *const FearType,
    nparams: u32,
    returns: FearType,
) -> FearFuncId {
    let m = as_module(m);

    m.declare_function(
        &cstr(name),
        FunctionSignature {
            params: to_vec(params, nparams as usize)
                .iter()
                .map(|t| Type::from(*t))
                .collect(),
            returns: returns.into(),
        },
        Linkage::External,
        CallingConvention::C,
    )
}

/* ========================= FUNCTION DEF ========================= */

#[unsafe(no_mangle)]
pub unsafe extern "C" fn fearDefinitionCreate() -> *mut FearFunctionDef {
    Box::into_raw(Box::new(FunctionDef::new())) as *mut FearFunctionDef
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn fearDefinitionDispose(f: *mut FearFunctionDef) {
    if !f.is_null() {
        drop(Box::from_raw(f));
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn fearDefineFunction(
    m: *mut FearModule,
    id: FearFuncId,
    def: *const FearFunctionDef,
) {
    let m = as_module(m);
    let def = as_def(def as *mut FearFunctionDef);
    m.define_function(id, def.clone()).unwrap();
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn fearGetEntryBlock(f: *mut FearFunctionDef) -> FearBlockId {
    as_def(f).entry
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn fearCreateBlock(f: *mut FearFunctionDef) -> FearBlockId {
    as_def(f).new_block()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn fearCreateFuncParam(f: *mut FearFunctionDef, ty: FearType) -> FearBlockId {
    as_def(f).add_param(ty.into())
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn fearCreateBlockParam(
    f: *mut FearFunctionDef,
    block: FearBlockId,
    ty: FearType,
) -> FearBlockId {
    as_def(f).add_block_param(block, ty.into())
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn fearCreateIntConst(
    f: *mut FearFunctionDef,
    parent: FearBlockId,
    ty: FearType,
    val: i64,
) -> FearValueId {
    as_def(f).make_iconst(parent, ty.into(), val)
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
pub unsafe extern "C" fn fearCreateSignedDiv(
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
pub unsafe extern "C" fn fearCreateSignedRem(
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

/* ========================= FLOAT ========================= */

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

#[unsafe(no_mangle)]
pub unsafe extern "C" fn fearCreateRet(
    f: *mut FearFunctionDef,
    parent: FearBlockId,
    v: FearValueId,
) {
    as_def(f).make_ret(parent, Some(v))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn fearCreateRetVoid(f: *mut FearFunctionDef, parent: FearBlockId) {
    as_def(f).make_ret(parent, None)
}
