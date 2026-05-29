use crate::{types::*, *};

/// Allocates a new empty `Module` on the heap and returns its raw pointer. Ownership is transferred to C.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn fearModuleCreate(name: *const c_char) -> *mut FearModule {
    let name = cstr(name);
    log::trace!("creating module with name {}", name);
    let m = Module::new(name);
    Box::into_raw(Box::new(m)) as *mut FearModule
}

/// Deserializes a complete `Module` structure from an input file descriptor. Returns null on error.
#[cfg(feature = "binary-ir")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn fearReadBinaryFromFile(fd: c_int) -> *mut FearModule {
    let file = unsafe { std::fs::File::from_raw_fd(fd) };
    match fear::binary::read::<fear::ssa::Module, _>(file) {
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
pub unsafe extern "C" fn fearModuleOptimize(m: *mut FearModule, level: FearOptLevel) -> u32 {
    let m = as_module(m);
    let map = m.optimize(OptLevel::from(level), false);
    let total_passes: usize = map.values().map(|result| result.passes.len()).sum();
    total_passes as u32
}

/// Runs internal optimization passes (e.g., dead code elimination, simplification) over the module's IR.
/// Uses multilevel pipeline
/// Return total passes count (exclude multilevel passes)
#[unsafe(no_mangle)]
pub unsafe extern "C" fn fearModuleOptimizeMultilevel(
    m: *mut FearModule,
    level: FearOptLevel,
) -> u32 {
    let m = as_module(m);
    let map = m.optimize(OptLevel::from(level), true);
    let total_passes: usize = map.values().map(|result| result.passes.len()).sum();
    total_passes as u32
}

/// Verify module
/// Return verify errors count.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn fearModuleVerify(m: *mut FearModule) -> u32 {
    let m = as_module(m);
    let res = m.verify();
    if let Err(e) = res {
        // errors count
        return e.len() as u32;
    }
    // not a error, zero code
    0
}
