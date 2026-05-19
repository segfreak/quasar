#![allow(clippy::missing_safety_doc, unsafe_op_in_unsafe_fn)]

use std::ffi::c_int;
use std::io::Write;
use std::os::fd::FromRawFd;
use std::os::raw::c_char;
use std::{ffi::CStr, ptr};

use fear::{
    binary,
    compiler::{self, Backend, CompilerConfig, OutputType},
    ir::*,
    types::{CallingConvention, FunctionSignature, Linkage, OptLevel, Type},
};

use target_lexicon::Triple;

/// Opaque wrapper for the internal `Module` struct, exposed to C as a raw pointer.
#[repr(C)]
pub struct FearModule {
    __: [i8; 0],
}

/// Opaque wrapper for the internal `FunctionDef` struct, used to build function bodies.
#[repr(C)]
pub struct FearFunctionDef {
    __: [i8; 0],
}

// Type aliases representing internal IR identifiers for functions, basic blocks, and SSA values.
pub type FearFuncId = u32;
pub type FearBlockId = u32;
pub type FearValueId = u32;

/// Supported types in the Fear IR type system.
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

/// Optimization levels mapping directly to compiler backend passes.
#[repr(C)]
#[derive(Clone, Copy)]
pub enum FearOptLevel {
    FearOptLevelNone,
    FearOptLevelDefault,
    FearOptLevelFull,
}

/// Calling conventions supported for function declarations and calls.
#[repr(C)]
#[derive(Clone, Copy)]
pub enum FearCallConv {
    FearCallConvC,
    FearCallConvSysV,
    FearCallConvMsAbi,
}

/// Linkage types specifying visibility and resolution rules for symbols.
#[repr(C)]
#[derive(Clone, Copy)]
pub enum FearLinkage {
    FearLinkageExternal,
    FearLinkageInternal,
    FearLinkageWeak,
}

// --- Conversions from C-compatible enums to native Rust compiler types ---

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
            FearOptLevel::FearOptLevelNone => OptLevel::None,
            FearOptLevel::FearOptLevelDefault => OptLevel::Default,
            FearOptLevel::FearOptLevelFull => OptLevel::Full,
        }
    }
}

impl From<FearCallConv> for CallingConvention {
    fn from(t: FearCallConv) -> Self {
        match t {
            FearCallConv::FearCallConvC => CallingConvention::C,
            FearCallConv::FearCallConvSysV => CallingConvention::SystemV,
            FearCallConv::FearCallConvMsAbi => CallingConvention::MicrosoftAbi,
        }
    }
}

impl From<FearLinkage> for Linkage {
    fn from(t: FearLinkage) -> Self {
        match t {
            FearLinkage::FearLinkageExternal => Linkage::External,
            FearLinkage::FearLinkageInternal => Linkage::Internal,
            FearLinkage::FearLinkageWeak => Linkage::Weak,
        }
    }
}

// --- Internal Helper Functions ---

/// Converts a raw C-string pointer into an owned Rust `String`.
fn cstr(s: *const c_char) -> String {
    unsafe { CStr::from_ptr(s).to_string_lossy().into_owned() }
}

/// Safely copies data from a raw array pointer into a standard Rust `Vec`.
unsafe fn to_vec<T: Clone>(data: *const T, nelem: usize) -> Vec<T> {
    if data.is_null() || nelem == 0 {
        return Vec::new();
    }
    std::slice::from_raw_parts(data, nelem).to_vec()
}

/// Casts a raw C pointer back into a mutable reference to the core `Module`.
unsafe fn as_module(m: *mut FearModule) -> &'static mut Module {
    &mut *(m as *mut Module)
}

/// Casts a raw C pointer back into a mutable reference to the core `FunctionDef`.
unsafe fn as_def(f: *mut FearFunctionDef) -> &'static mut FunctionDef {
    &mut *(f as *mut FunctionDef)
}

// --- C-API Exported Functions (FFI Boundary) ---

/// Checks if the LLVM lowering backend is compiled into the library.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn fearLoweringHasLLVM() -> bool {
    fear::lowering::has_llvm()
}

/// Checks if the Cranelift lowering backend is compiled into the library.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn fearLoweringHasCranelift() -> bool {
    fear::lowering::has_cranelift()
}

/// Compiles a `FearModule` into a native machine object file via Cranelift, streaming to a raw file descriptor.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn fearEmitCraneliftObjectToFile(
    m: *mut FearModule,
    opt: FearOptLevel,
    fd: c_int,
) -> c_int {
    let m = as_module(m);
    let config = CompilerConfig {
        backend: Backend::Cranelift,
        output_type: OutputType::Object,
        triple: Triple::host(),
        opt_level: OptLevel::from(opt),
    };
    let file = unsafe { std::fs::File::from_raw_fd(fd) };
    match compiler::compile_module(m, &config, file) {
        Ok(_) => 0,
        Err(e) => {
            log::error!("compile error: {}", e);
            1
        }
    }
}

/// Allocates a new empty `Module` on the heap and returns its raw pointer. Ownership is transferred to C.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn fearModuleCreate(name: *const c_char) -> *mut FearModule {
    let name = cstr(name);
    log::trace!("creating module with name {}", name);
    let m = Module::new(name);
    Box::into_raw(Box::new(m)) as *mut FearModule
}

/// Deserializes a complete `Module` structure from an input file descriptor. Returns null on error.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn fearReadBinaryFromFile(fd: c_int) -> *mut FearModule {
    let file = unsafe { std::fs::File::from_raw_fd(fd) };
    match fear::binary::read::<fear::ir::Module, _>(file) {
        Ok(m) => Box::into_raw(Box::new(m)) as *mut FearModule,
        Err(e) => {
            log::error!("cannot read binary module from fd({})", fd);
            log::error!("{}", e);
            ptr::null_mut()
        }
    }
}

/// Reclaims ownership of a `FearModule` pointer and drops it, freeing all associated heap memory.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn fearModuleDispose(m: *mut FearModule) {
    if !m.is_null() {
        drop(Box::from_raw(m));
    }
}

/// Runs internal optimization passes (e.g., dead code elimination, simplification) over the module's IR.
/// Return total passes count
#[unsafe(no_mangle)]
pub unsafe extern "C" fn fearModuleOptimize(m: *mut FearModule) -> u32 {
    let m = as_module(m);
    let map = m.optimize();
    let total_passes: usize = map.values().map(|result| result.passes.len()).sum();
    total_passes as u32
}

/// Writes a readable, plain-text representation of the module's IR into a file descriptor.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn fearDumpToFile(m: *mut FearModule, fd: c_int) {
    let m = as_module(m);
    let s = m.dump();
    let mut file = unsafe { std::fs::File::from_raw_fd(fd) };
    let _ = file.write_all(s.as_bytes());
    std::mem::forget(file); // Prevents early closure of the file descriptor owned by the host
}

/// Serializes the module into the compiler's native binary format and outputs it to a file descriptor.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn fearBinaryDumpToFile(m: *mut FearModule, fd: c_int) {
    let m = as_module(m);
    let file = unsafe { std::fs::File::from_raw_fd(fd) };
    if let Err(e) = binary::write(m, file) {
        log::error!("cannot write binary module into fd({})", fd);
        log::error!("{}", e);
    }
}

/// Declares a function prototype inside the module. Returns a unique function identifier.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn fearDeclareFunction(
    m: *mut FearModule,
    name: *const c_char,
    params: *const FearType,
    nparams: u32,
    returns: FearType,
    linkage: FearLinkage,
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
        Linkage::from(linkage),
        CallingConvention::C,
    )
}

/// Modifies the calling convention of a previously declared function.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn fearFunctionSetCC(m: *mut FearModule, id: FearFuncId, cc: FearCallConv) {
    let m = as_module(m);

    if let Some(f) = m.get_function_mut(id as FuncId) {
        f.calling_convention = CallingConvention::from(cc);
    }
}

/// Allocates an empty `FunctionDef` on the heap to start building an SSA function body.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn fearDefinitionCreate() -> *mut FearFunctionDef {
    Box::into_raw(Box::new(FunctionDef::new())) as *mut FearFunctionDef
}

/// Deallocates a `FunctionDef` from heap memory if it wasn't consumed by `fearDefineFunction`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn fearDefinitionDispose(f: *mut FearFunctionDef) {
    if !f.is_null() {
        drop(Box::from_raw(f));
    }
}

/// Binds a complete function definition (cloned) (`FunctionDef`) to a declared function ID within the module.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn fearDefineFunction(
    m: *mut FearModule,
    id: FearFuncId,
    def: *const FearFunctionDef,
) {
    let m = as_module(m);
    let def = as_def(def as *mut FearFunctionDef);
    if let Err(e) = m.define_function(id, def.clone()) {
        log::error!("{}", e);
    }
}

/// Gets the identifier of the entry basic block automatically created for a function definition.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn fearGetEntryBlock(f: *mut FearFunctionDef) -> FearBlockId {
    as_def(f).entry
}

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
) {
    as_def(f).make_call(parent, ty.into(), func, to_vec(args, nargs as usize));
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
