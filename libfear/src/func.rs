use crate::{types::*, *};

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
    as_def(f).get_entry()
}

/// Sets the identifier of the entry basic block.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn fearSetEntryBlock(f: *mut FearFunctionDef, block: FearBlockId) {
    *as_def(f).get_entry_mut() = block;
}
