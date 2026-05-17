use std::ffi::{CStr, CString};
use std::os::raw::c_char;

use crate::ir::*;
use fearcore::{target::*, *};

fn cstr(s: *const c_char) -> String {
    unsafe { CStr::from_ptr(s).to_string_lossy().into_owned() }
}

unsafe fn to_vec<T>(data: *const T, nelem: usize) -> Vec<T>
where
    T: Clone,
{
    if data.is_null() || nelem == 0 {
        return Vec::new();
    }
    let slice = unsafe { std::slice::from_raw_parts(data, nelem) };
    slice.to_vec()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn fear_module_new(name: *const c_char) -> *mut Module {
    let m = Module::new(cstr(name));
    Box::into_raw(Box::new(m))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn fear_module_free(m: *mut Module) {
    if !m.is_null() {
        unsafe { drop(Box::from_raw(m)) }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn fear_declare_function(
    m: *mut Module,
    name: *const c_char,
    params: *const Type,
    nparams: u32,
    returns: Type,
) -> FuncId {
    let m = unsafe { m.as_mut().unwrap() };
    m.declare_function(
        &cstr(name),
        FunctionSignature {
            params: unsafe { to_vec::<Type>(params, nparams as usize) },
            returns,
        },
        Linkage::External,
        CallingConvention::C,
    )
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn fear_define_function(m: *mut Module, id: FuncId, def: *const FunctionDef) {
    let m = unsafe { m.as_mut().unwrap() };
    let def = unsafe { def.as_ref().unwrap().clone() };
    m.define_function(id, def).unwrap();
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn fear_definition_new() -> *mut FunctionDef {
    let f = FunctionDef::new();
    Box::into_raw(Box::new(f))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn fear_definition_free(f: *mut FunctionDef) {
    if !f.is_null() {
        unsafe { drop(Box::from_raw(f)) }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn fear_block_entry(f: *mut FunctionDef) -> BlockId {
    let f = unsafe { f.as_mut().unwrap() };
    f.entry
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn fear_block_new(f: *mut FunctionDef) -> BlockId {
    let f = unsafe { f.as_mut().unwrap() };
    f.new_block()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn fear_iconst(
    f: *mut FunctionDef,
    parent: BlockId,
    ty: Type,
    iconst: i64,
) -> ValueId {
    let f = unsafe { f.as_mut().unwrap() };
    f.make_iconst(parent, ty, iconst)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn fear_add(
    f: *mut FunctionDef,
    parent: BlockId,
    ty: Type,
    left: ValueId,
    right: ValueId,
) -> ValueId {
    let f = unsafe { f.as_mut().unwrap() };
    f.make_add(parent, ty, left, right)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn fear_sub(
    f: *mut FunctionDef,
    parent: BlockId,
    ty: Type,
    left: ValueId,
    right: ValueId,
) -> ValueId {
    let f = unsafe { f.as_mut().unwrap() };
    f.make_sub(parent, ty, left, right)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn fear_mul(
    f: *mut FunctionDef,
    parent: BlockId,
    ty: Type,
    left: ValueId,
    right: ValueId,
) -> ValueId {
    let f = unsafe { f.as_mut().unwrap() };
    f.make_mul(parent, ty, left, right)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn fear_sdiv(
    f: *mut FunctionDef,
    parent: BlockId,
    ty: Type,
    left: ValueId,
    right: ValueId,
) -> ValueId {
    let f = unsafe { f.as_mut().unwrap() };
    f.make_div(parent, true, ty, left, right)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn fear_udiv(
    f: *mut FunctionDef,
    parent: BlockId,
    ty: Type,
    left: ValueId,
    right: ValueId,
) -> ValueId {
    let f = unsafe { f.as_mut().unwrap() };
    f.make_div(parent, false, ty, left, right)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn fear_srem(
    f: *mut FunctionDef,
    parent: BlockId,
    ty: Type,
    left: ValueId,
    right: ValueId,
) -> ValueId {
    let f = unsafe { f.as_mut().unwrap() };
    f.make_rem(parent, true, ty, left, right)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn fear_urem(
    f: *mut FunctionDef,
    parent: BlockId,
    ty: Type,
    left: ValueId,
    right: ValueId,
) -> ValueId {
    let f = unsafe { f.as_mut().unwrap() };
    f.make_rem(parent, false, ty, left, right)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn fear_fadd(
    f: *mut FunctionDef,
    parent: BlockId,
    ty: Type,
    left: ValueId,
    right: ValueId,
) -> ValueId {
    let f = unsafe { f.as_mut().unwrap() };
    f.make_fadd(parent, ty, left, right)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn fear_fsub(
    f: *mut FunctionDef,
    parent: BlockId,
    ty: Type,
    left: ValueId,
    right: ValueId,
) -> ValueId {
    let f = unsafe { f.as_mut().unwrap() };
    f.make_fsub(parent, ty, left, right)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn fear_fmul(
    f: *mut FunctionDef,
    parent: BlockId,
    ty: Type,
    left: ValueId,
    right: ValueId,
) -> ValueId {
    let f = unsafe { f.as_mut().unwrap() };
    f.make_fmul(parent, ty, left, right)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn fear_fdiv(
    f: *mut FunctionDef,
    parent: BlockId,
    ty: Type,
    left: ValueId,
    right: ValueId,
) -> ValueId {
    let f = unsafe { f.as_mut().unwrap() };
    f.make_fdiv(parent, ty, left, right)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn fear_frem(
    f: *mut FunctionDef,
    parent: BlockId,
    ty: Type,
    left: ValueId,
    right: ValueId,
) -> ValueId {
    let f = unsafe { f.as_mut().unwrap() };
    f.make_frem(parent, ty, left, right)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn fear_jump(
    f: *mut FunctionDef,
    parent: BlockId,
    target: BlockId,
    params: *const ValueId,
    nparams: u32,
) {
    let f = unsafe { f.as_mut().unwrap() };
    f.make_jump(parent, target, unsafe { to_vec(params, nparams as usize) })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn fear_jumpif(
    f: *mut FunctionDef,
    parent: BlockId,
    cond: ValueId,
    then_target: BlockId,
    then_params: *const ValueId,
    then_nparams: u32,
    else_target: BlockId,
    else_params: *const ValueId,
    else_nparams: u32,
) {
    let f = unsafe { f.as_mut().unwrap() };
    f.make_jumpif(
        parent,
        cond,
        then_target,
        unsafe { to_vec(then_params, then_nparams as usize) },
        else_target,
        unsafe { to_vec(else_params, else_nparams as usize) },
    )
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn fear_ret(f: *mut FunctionDef, parent: BlockId, value: ValueId) {
    let f = unsafe { f.as_mut().unwrap() };
    f.make_ret(parent, Some(value))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn fear_ret_void(f: *mut FunctionDef, parent: BlockId) {
    let f = unsafe { f.as_mut().unwrap() };
    f.make_ret(parent, None)
}
