use std::collections::HashMap;

use crate::ssa::*;
use crate::types::Linkage;

#[derive(thiserror::Error, Debug, Clone, PartialEq, Eq)]
pub enum LinkError {
    /// external symbol is not resolved
    #[error("undefined symbol: `{0}`")]
    UndefinedSymbol(String),

    /// multiple strong candidates
    #[error("duplicate definition of `{0}`")]
    DuplicateDefinition(String),

    /// try to redefine a symbol with an incompatible signature
    #[error("signature mismatch for `{name}`: existing `{existing}`, incoming `{incoming}`")]
    SignatureMismatch {
        name: String,
        existing: String,
        incoming: String,
    },
}

#[derive(Debug, Clone)]
pub enum LinkWarning {
    WeakOverridden { name: String },
    WeakKept { name: String },
}

#[derive(Debug, Clone)]
pub struct LinkResult {
    pub module: Module,
    pub warnings: Vec<LinkWarning>,
}

#[derive(Debug, Clone)]
pub struct Linker {
    output: Module,
    symbol_table: HashMap<String, FuncId>,
    warnings: Vec<LinkWarning>,
}

impl Linker {
    pub fn new(output_name: impl Into<String>) -> Self {
        Self {
            output: Module::new(output_name),
            symbol_table: HashMap::new(),
            warnings: Vec::new(),
        }
    }

    pub fn link_module(&mut self, module: Module) -> Result<(), LinkError> {
        let mut id_remap: HashMap<FuncId, FuncId> = HashMap::new();

        for (incoming_id, func) in &module.functions {
            let name = &func.name;

            match self.symbol_table.get(name).copied() {
                None => {
                    let out_id = self.output.declare_function(
                        name,
                        func.signature.clone(),
                        func.linkage,
                        func.calling_convention,
                    );
                    self.symbol_table.insert(name.clone(), out_id);
                    id_remap.insert(*incoming_id, out_id);
                }

                Some(existing_out_id) => {
                    let existing_sig = &self.output.functions[&existing_out_id].signature;
                    if existing_sig != &func.signature {
                        return Err(LinkError::SignatureMismatch {
                            name: name.clone(),
                            existing: format!("{:?}", existing_sig),
                            incoming: format!("{:?}", func.signature),
                        });
                    }

                    id_remap.insert(*incoming_id, existing_out_id);
                }
            }
        }

        for (incoming_id, func) in module.functions {
            let out_id = id_remap[&incoming_id];
            let name = func.name.clone();

            let Some(incoming_def) = func.get_definition().cloned() else {
                continue;
            };

            let existing_has_def = self.output.functions[&out_id].get_definition().is_some();

            if existing_has_def {
                let existing_linkage = self.output.functions[&out_id].linkage;
                let incoming_linkage = func.linkage;

                match (existing_linkage, incoming_linkage) {
                    (
                        Linkage::External | Linkage::Internal,
                        Linkage::External | Linkage::Internal,
                    ) => {
                        return Err(LinkError::DuplicateDefinition(name));
                    }

                    (Linkage::External | Linkage::Internal, Linkage::Weak) => {
                        self.warnings
                            .push(LinkWarning::WeakOverridden { name: name.clone() });
                    }

                    (Linkage::Weak, Linkage::External | Linkage::Internal) => {
                        self.warnings
                            .push(LinkWarning::WeakOverridden { name: name.clone() });
                        let remapped = remap_function_def(incoming_def, &id_remap);
                        self.output
                            .set_definition(out_id, remapped)
                            .expect("set_definition failed after clearing");
                    }
                    (Linkage::Weak, Linkage::Weak) => {
                        self.warnings
                            .push(LinkWarning::WeakKept { name: name.clone() });
                    }
                }
            } else {
                let remapped = remap_function_def(incoming_def, &id_remap);
                self.output
                    .define_function(out_id, remapped)
                    .expect("define_function failed on fresh slot");

                if let Some(f) = self.output.get_function_mut(out_id) {
                    f.linkage = func.linkage;
                }
            }
        }

        Ok(())
    }

    pub fn finish(self) -> Result<LinkResult, Vec<LinkError>> {
        let mut errors = Vec::new();

        for (name, &out_id) in &self.symbol_table {
            let func = &self.output.functions[&out_id];
            if func.get_definition().is_none() && func.linkage == Linkage::External {
                errors.push(LinkError::UndefinedSymbol(name.clone()));
            }
        }

        if !errors.is_empty() {
            return Err(errors);
        }

        Ok(LinkResult {
            module: self.output,
            warnings: self.warnings,
        })
    }
}

fn remap_function_def(mut def: FunctionDef, id_remap: &HashMap<FuncId, FuncId>) -> FunctionDef {
    for inst in def.get_insts_mut().values_mut() {
        #[allow(clippy::collapsible_if)]
        if let InstKind::Call(ref mut fid) = inst.kind {
            if let Some(&new_id) = id_remap.get(fid) {
                *fid = new_id;
            }
        }
    }
    def
}

pub fn link(
    output_name: impl Into<String>,
    modules: impl IntoIterator<Item = Module>,
) -> Result<LinkResult, Vec<LinkError>> {
    let mut linker = Linker::new(output_name);
    for module in modules {
        linker.link_module(module).map_err(|e| vec![e])?;
    }
    linker.finish()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ssa::{FunctionDef, Module};
    use crate::types::{CallingConvention, FunctionSignature, Linkage, Type};

    fn make_empty_def() -> FunctionDef {
        let mut def = FunctionDef::new();
        def.make_ret(def.get_entry(), None);
        def
    }

    fn sig(ret: Type, params: Vec<Type>) -> FunctionSignature {
        FunctionSignature {
            params,
            returns: ret,
        }
    }

    fn simple_mod(name: &str, func_name: &str, linkage: Linkage, has_def: bool) -> Module {
        let mut m = Module::new(name);
        let s = sig(Type::Void, vec![]);
        if has_def {
            let func = crate::ssa::Function::definition(
                func_name,
                s,
                linkage,
                CallingConvention::default(),
                make_empty_def(),
            );
            m.add_function(func);
        } else {
            m.declare_function(func_name, s, linkage, CallingConvention::default());
        }
        m
    }

    #[test]
    fn test_simple_link() {
        let m1 = simple_mod("a", "foo", Linkage::External, true);
        let m2 = simple_mod("b", "bar", Linkage::External, true);
        let result = link("out", [m1, m2]).expect("link failed");
        assert!(result.module.lookup_function("foo").is_some());
        assert!(result.module.lookup_function("bar").is_some());
    }

    #[test]
    fn test_declaration_resolved() {
        // m1 only declares `foo`, m2 defines it.
        let m1 = simple_mod("a", "foo", Linkage::External, false);
        let m2 = simple_mod("b", "foo", Linkage::External, true);
        let result = link("out", [m1, m2]).expect("link failed");
        let fid = result.module.lookup_function("foo").unwrap();
        assert!(result
            .module
            .get_function(fid)
            .unwrap()
            .get_definition()
            .is_some());
    }

    #[test]
    fn test_duplicate_strong_definition_error() {
        let m1 = simple_mod("a", "foo", Linkage::External, true);
        let m2 = simple_mod("b", "foo", Linkage::External, true);
        let err = link("out", [m1, m2]).unwrap_err();
        assert!(matches!(err[0], LinkError::DuplicateDefinition(_)));
    }

    #[test]
    fn test_undefined_symbol_error() {
        let m1 = simple_mod("a", "foo", Linkage::External, false);
        let err = link("out", [m1]).unwrap_err();
        assert!(matches!(err[0], LinkError::UndefinedSymbol(_)));
    }

    #[test]
    fn test_weak_overridden_by_strong() {
        let m1 = simple_mod("a", "foo", Linkage::Weak, true);
        let m2 = simple_mod("b", "foo", Linkage::External, true);
        let result = link("out", [m1, m2]).expect("link failed");
        // strong wins, one warning
        assert_eq!(result.warnings.len(), 1);
        assert!(matches!(
            result.warnings[0],
            LinkWarning::WeakOverridden { .. }
        ));
    }

    #[test]
    fn test_weak_kept_when_no_strong() {
        let m1 = simple_mod("a", "foo", Linkage::Weak, true);
        let m2 = simple_mod("b", "foo", Linkage::Weak, true);
        let result = link("out", [m1, m2]).expect("link failed");
        assert_eq!(result.warnings.len(), 1);
        assert!(matches!(result.warnings[0], LinkWarning::WeakKept { .. }));
    }

    #[test]
    fn test_cross_module_call_remap() {
        // m1 defines `callee`, m2 defines `caller` which calls `callee`.
        let mut m1 = Module::new("m1");
        let callee_sig = sig(Type::Int64, vec![]);
        let callee_id = m1.declare_function(
            "callee",
            callee_sig.clone(),
            Linkage::External,
            CallingConvention::default(),
        );
        let mut callee_def = FunctionDef::new();
        let entry = callee_def.get_entry();
        let c = callee_def.make_int_const(entry, Type::Int64, 42);
        callee_def.make_ret(entry, Some(c));
        m1.define_function(callee_id, callee_def).unwrap();

        let mut m2 = Module::new("m2");
        // Declaration of `callee` in m2 – will get a *different* local FuncId.
        let callee_decl_id = m2.declare_function(
            "callee",
            callee_sig,
            Linkage::External,
            CallingConvention::default(),
        );

        let caller_sig = sig(Type::Void, vec![]);
        let caller_id = m2.declare_function(
            "caller",
            caller_sig,
            Linkage::External,
            CallingConvention::default(),
        );
        let mut caller_def = FunctionDef::new();
        let entry = caller_def.get_entry();
        // Call uses the *local* id of `callee` inside m2.
        caller_def.make_call(entry, Type::Int64, callee_decl_id, vec![]);
        caller_def.make_ret(entry, None);
        m2.define_function(caller_id, caller_def).unwrap();

        let result = link("out", [m1, m2]).expect("link failed");

        // After linking, the call inside `caller` must point to the unified
        // output FuncId for `callee` – not the stale m2-local id.
        let out_callee = result.module.lookup_function("callee").unwrap();
        let out_caller = result.module.lookup_function("caller").unwrap();
        let def = result
            .module
            .get_function(out_caller)
            .unwrap()
            .get_definition()
            .unwrap();

        let has_correct_call = def
            .get_insts()
            .values()
            .any(|inst| matches!(inst.kind, InstKind::Call(fid) if fid == out_callee));
        assert!(
            has_correct_call,
            "cross-module call was not remapped correctly"
        );
    }
}
