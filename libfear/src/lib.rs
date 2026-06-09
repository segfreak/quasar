#![allow(clippy::missing_safety_doc, unsafe_op_in_unsafe_fn)]

pub mod builder;
pub mod func;
pub mod module;
pub mod types;

use types::*;

use std::ffi::{c_int, CString};
use std::io::Write;
use std::os::fd::FromRawFd;
use std::os::raw::c_char;
use std::{ffi::CStr, ptr};

use fear::{
    binary,
    compiler::{self, Backend, CompilerConfig, OutputType},
    ssa::*,
    types::{CallingConvention, FunctionSignature, Linkage, OptLevel, Type},
};

use target_lexicon::Triple;

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

/// Checks if the backend is supported in this library build.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn fearHasBackend(backend: FearBackend) -> bool {
    fear::compiler::has_backend(Backend::from(backend))
}

/// Initialising logging system
#[unsafe(no_mangle)]
pub unsafe extern "C" fn fearInitLogging() -> i32 {
    pretty_env_logger::try_init().is_err() as i32
}

/// Select any backend
/// Check by fearHasBackend()
#[unsafe(no_mangle)]
pub unsafe extern "C" fn fearSelectBackend() -> FearBackend {
    let t = if fearHasBackend(FearBackend::FearBackendLlvm) {
        FearBackend::FearBackendLlvm
    } else if fearHasBackend(FearBackend::FearBackendCranelift) {
        FearBackend::FearBackendCranelift
    } else {
        FearBackend::FearBackendDummy
    };

    if !fearHasBackend(t) {
        log::warn!("own backend is not yet implemented");
        log::warn!("this can be source of this error");
        log::error!("selected backend is not supported");
    }

    t
}

/// Select backend for output type object
/// Check by fearHasBackend()
#[unsafe(no_mangle)]
pub unsafe extern "C" fn fearSelectBackendForObject() -> FearBackend {
    if let Some(backend) = Backend::select_for(OutputType::Object) {
        FearBackend::from(backend)
    } else {
        log::warn!("cannot select backend for object");
        log::warn!("using dummy backend as fallback");
        FearBackend::FearBackendDummy
    }
}

/// Compiles a `FearModule` into a native machine object file via target backend, streaming to a raw file descriptor.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn fearEmitObject(
    m: *mut FearModule,
    backend: FearBackend,
    opt: FearOptLevel,
    fd: c_int,
) -> c_int {
    let m = as_module(m);
    let config = CompilerConfig {
        backend: Backend::from(backend),
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

/// Supported backends: FearBackendLlvm
/// Compiles a `FearModule` into a native machine assembly file via target backend, streaming to a raw file descriptor.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn fearEmitAssembly(
    m: *mut FearModule,
    backend: FearBackend,
    opt: FearOptLevel,
    fd: c_int,
) -> c_int {
    let m = as_module(m);
    let config = CompilerConfig {
        backend: Backend::from(backend),
        output_type: OutputType::Assembly,
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

/// Writes a readable, plain-text representation of the module's IR into a file descriptor.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn fearDumpToFile(m: *mut FearModule, fd: c_int) {
    let m = as_module(m);
    let s = m.dump();
    let mut file = unsafe { std::fs::File::from_raw_fd(fd) };
    let _ = file.write_all(s.as_bytes());
    std::mem::forget(file);
}

/// Writes a readable, plain-text representation of the module's IR into a C String.
/// Needs to free()
#[unsafe(no_mangle)]
pub unsafe extern "C" fn fearDumpToString(m: *mut FearModule) -> *mut c_char {
    let m = as_module(m);
    let s = m.dump();
    match CString::new(s) {
        Ok(c_str) => c_str.into_raw(),
        Err(_) => std::ptr::null_mut(),
    }
}

/// Frees rust-side allocated string
#[unsafe(no_mangle)]
pub unsafe extern "C" fn fearStringDispose(s: *mut c_char) {
    if !s.is_null() {
        unsafe {
            let _ = CString::from_raw(s);
        }
    }
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

/// Serializes the module into the compiler's native binary format and outputs it to a sized buffer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn fearBinaryDumpToBuffer(
    m: *mut FearModule,
    out_size: *mut usize,
) -> *mut u8 {
    let m = as_module(m);
    let mut buf = Vec::new();

    if let Err(e) = binary::write(m, &mut buf) {
        log::error!("cannot write binary module into buffer: {}", e);
        unsafe {
            *out_size = 0;
        }
        return std::ptr::null_mut();
    }

    unsafe {
        *out_size = buf.len();
    }
    let mut boxed_slice = buf.into_boxed_slice();
    let ptr = boxed_slice.as_mut_ptr();
    std::mem::forget(boxed_slice);
    ptr
}

/// Frees rust-side allocated buffer
#[unsafe(no_mangle)]
pub unsafe extern "C" fn fearBufferDispose(ptr: *mut u8, size: usize) {
    if !ptr.is_null() {
        unsafe {
            let _ = Vec::from_raw_parts(ptr, size, size);
        }
    }
}
