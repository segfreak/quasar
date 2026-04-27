use std::fs;

use quassa::{ir::*, target::CallingConvention};

fn foo_def() -> FunctionDef {
    let mut fun = FunctionDef::new();
    let a0 = fun.add_param(Type::I16);
    let a1 = fun.add_param(Type::I16);
    let v0 = fun.make_iconst(fun.entry, Type::I32, 42);
    let v1 = fun.make_iconst(fun.entry, Type::I32, 2);
    let v2 = fun.make_mul(fun.entry, fun.get_type(v0), vec![v0, v1]).1;
    let v3 = fun.make_div(fun.entry, fun.get_type(v2), vec![v2, a0]).1;
    let v4 = fun.make_sub(fun.entry, fun.get_type(v2), vec![v3, a0]).1;
    let v5 = fun.make_sub(fun.entry, fun.get_type(v2), vec![v4, a1]).1;
    fun.append_inst(fun.entry, InstKind::Ret, Type::Void, vec![v5]);
    fun
}

fn bar_def() -> FunctionDef {
    let mut fun = FunctionDef::new();
    let a0 = fun.add_param(Type::I16);
    let a1 = fun.add_param(Type::I16);
    let v0 = fun.make_iconst(fun.entry, Type::I32, 42);
    let v1 = fun.make_iconst(fun.entry, Type::I32, 2);
    let v2 = fun.make_mul(fun.entry, fun.get_type(v0), vec![v0, v1]).1;
    let v3 = fun.make_div(fun.entry, fun.get_type(v2), vec![v2, a0]).1;
    fun.append_inst(fun.entry, InstKind::Ret, Type::Void, vec![v3]);
    fun
}

fn main() {
    let mut m = Module::new("quasar");
    let mfoo = m.declare_function(
        "mfoo",
        FunctionSignature::new(vec![Type::I16, Type::I16], Type::I32),
        Linkage::default(),
        CallingConvention::default(),
    );
    let mbar = m.declare_function(
        "mbar",
        FunctionSignature::new(vec![Type::I16, Type::I16], Type::I32),
        Linkage::default(),
        CallingConvention::default(),
    );
    m.define_function(mfoo, foo_def())
        .expect("define_function error");
    m.define_function(mbar, bar_def())
        .expect("define_function error");
    m.optimize();
    m.verify().expect("verify error");
    fs::write("quasar.dot", m.dump_dot()).expect("fs::write error");
}
