use std::fs;

use quassa::{ir::*, target::CallingConvention};

fn foo_def() -> FunctionDef {
    let mut fun = FunctionDef::new();
    let b0 = fun.entry;
    let a0 = fun.add_param(Type::I32);
    let a1 = fun.add_param(Type::I32);
    let v0 = fun.make_iconst(b0, Type::I32, 42);
    let v1 = fun.make_iconst(b0, Type::I32, 2);
    let v2 = fun.make_mul(b0, fun.get_type(v0), v0, v1).1;
    let v3 = fun.make_div(b0, fun.get_type(v2), v2, a0).1;
    let v4 = fun.make_sub(b0, fun.get_type(v2), v3, a0).1;
    let v5 = fun.make_sub(b0, fun.get_type(v2), v4, a1).1;
    fun.make_ret(b0, Some(v5));
    fun
}

fn bar_def(mfoo: FuncId) -> FunctionDef {
    let mut fun = FunctionDef::new();
    let _a0 = fun.add_param(Type::I32);
    let b0 = fun.entry;
    let v0 = fun.make_iconst(b0, Type::I32, 42);
    let v1 = fun.make_iconst(b0, Type::I32, 2);
    let v2 = fun.make_mul(b0, fun.get_type(v0), v0, v1).1;
    let v3 = fun.make_div(b0, fun.get_type(v2), v2, v0).1;
    let b1 = fun.new_block();
    fun.make_jump(b0, b1, vec![v3]);
    let v4 = fun.add_block_param(b1, Type::I32);
    let v5 = fun.make_iconst(b1, Type::I32, 2);
    let v6 = fun.make_call(b1, Type::I32, mfoo, vec![v4, v5]).1;
    let v7 = fun.make_mul(b1, Type::I32, v5, v6).1;
    fun.make_ret(b1, Some(v7));
    fun
}

fn baz_def() -> FunctionDef {
    let mut fun = FunctionDef::new();
    let b0 = fun.entry;
    let v0 = fun.make_alloca(b0, Type::I32).1;
    fun.make_alloca(b0, Type::I32);

    let v2 = fun.make_iconst(b0, Type::I32, 42);
    let v3 = fun.make_iconst(b0, Type::I32, 42 * 2);

    fun.make_store(b0, false, v0, v2);
    fun.make_store(b0, false, v0, v3);
    fun.make_ret(b0, None);
    fun
}

fn opt_def() -> FunctionDef {
    let mut fdef = FunctionDef::new();

    let entry = fdef.entry;

    let x = fdef.add_param(Type::I32);

    // x * 2
    let two = fdef.make_iconst(entry, Type::I32, 2);
    let x2 = fdef.make_mul(entry, Type::I32, x, two).1;

    // (x * 2) + 0  -> dead + folding target
    let zero = fdef.make_iconst(entry, Type::I32, 0);
    let x2_plus0 = fdef.make_add(entry, Type::I32, x2, zero).1;

    // condition: (x * 2 + 0) > 10
    let ten = fdef.make_iconst(entry, Type::I32, 10);
    let cond = fdef.make_cmp(entry, CmpKind::Gt, x2_plus0, ten);

    let then_bb = fdef.new_block();
    let else_bb = fdef.new_block();

    // THEN: (x * 2) + (x * 2)
    let then_x2_a = fdef.make_mul(then_bb, Type::I32, x, two);
    let then_x2_b = fdef.make_mul(then_bb, Type::I32, x, two);
    let then_res = fdef.make_add(then_bb, Type::I32, then_x2_a.1, then_x2_b.1);
    fdef.make_ret(then_bb, Some(then_res.1));

    // ELSE: x * 2
    let else_x2 = fdef.make_mul(else_bb, Type::I32, x, two);
    fdef.make_ret(else_bb, Some(else_x2.1));

    // entry jump
    fdef.make_jumpif(entry, cond.1, then_bb, vec![], else_bb, vec![]);

    fdef
}

fn main() {
    pretty_env_logger::init();
    let mut m = Module::new("quasar");
    let mfoo = m.declare_function(
        "foo",
        FunctionSignature::new(vec![Type::I32, Type::I32], Type::I32),
        Linkage::default(),
        CallingConvention::default(),
    );
    let mbar = m.declare_function(
        "bar",
        FunctionSignature::new(vec![Type::I32], Type::I32),
        Linkage::default(),
        CallingConvention::default(),
    );
    let mbaz = m.declare_function(
        "baz",
        FunctionSignature::new(vec![], Type::Void),
        Linkage::default(),
        CallingConvention::default(),
    );
    let mopt = m.declare_function(
        "opt",
        FunctionSignature::new(vec![Type::I32], Type::I32),
        Linkage::default(),
        CallingConvention::default(),
    );
    m.define_function(mfoo, foo_def())
        .expect("define_function error");
    m.define_function(mbar, bar_def(mfoo))
        .expect("define_function error");
    m.define_function(mbaz, baz_def())
        .expect("define_function error");
    m.define_function(mopt, opt_def())
        .expect("define_function error");
    m.verify().expect("pre-opt verify error");
    m.optimize();
    m.verify().expect("post-opt verify error");
    fs::write("quasar.dot", m.dump_dot()).expect("fs::write error");
}
