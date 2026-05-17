use std::fs;

use inkwell::context::Context;
use mirssa::{ir::*, lowering::llvm::LlvmLowerer, parser::MirParser};
use pest::Parser;
use quasar::{target::CallingConvention, *};

fn foo_def() -> FunctionDef {
    let mut fun = FunctionDef::new();
    let b0 = fun.entry;
    let a0 = fun.add_param(Type::I32);
    let a1 = fun.add_param(Type::I32);
    let v0 = fun.make_iconst(b0, Type::I32, 42);
    let v1 = fun.make_iconst(b0, Type::I32, 2);
    let v2 = fun.make_mul(b0, fun.get_type(v0), v0, v1).1;
    let v3 = fun.make_div(b0, true, fun.get_type(v2), v2, a0).1;
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
    let v3 = fun.make_div(b0, true, fun.get_type(v2), v2, v0).1;
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

fn fib_def(fib_id: FuncId) -> FunctionDef {
    let mut f = FunctionDef::new();
    let entry = f.entry;

    // param n
    let n = f.add_param(Type::I32);

    // const 1
    let one = f.make_iconst(entry, Type::I32, 1);

    // n <= 1
    let cond = f.make_cmp(entry, CmpKind::Le, n, one).1;

    let then_bb = f.new_block();
    let else_bb = f.new_block();

    // if (n <= 1) goto then else else
    f.make_jumpif(entry, cond, then_bb, vec![], else_bb, vec![]);

    // THEN: return n
    f.make_ret(then_bb, Some(n));

    // ELSE:
    // n - 1
    let n_minus_1 = f.make_sub(else_bb, Type::I32, n, one).1;
    let call_fib_1 = f.make_call(else_bb, Type::I32, fib_id, vec![n_minus_1]).1;

    // n - 2
    let two = f.make_iconst(else_bb, Type::I32, 2);
    let n_minus_2 = f.make_sub(else_bb, Type::I32, n, two).1;
    let call_fib_2 = f.make_call(else_bb, Type::I32, fib_id, vec![n_minus_2]).1;

    // fib(n-1) + fib(n-2)
    let sum = f.make_add(else_bb, Type::I32, call_fib_1, call_fib_2).1;

    f.make_ret(else_bb, Some(sum));

    f
}

// tail-recursive factorial
fn fact_tr_def(fact_id: FuncId) -> FunctionDef {
    let mut f = FunctionDef::new();
    let entry = f.entry;

    let n = f.add_param(Type::I32);
    let acc = f.add_param(Type::I32);

    let one = f.make_iconst(entry, Type::I32, 1);

    let cond = f.make_cmp(entry, CmpKind::Le, n, one).1;

    let then_bb = f.new_block();
    let else_bb = f.new_block();

    f.make_jumpif(entry, cond, then_bb, vec![], else_bb, vec![]);

    f.make_ret(then_bb, Some(acc));

    let n_minus_1 = f.make_sub(else_bb, Type::I32, n, one).1;

    let acc_mul_n = f.make_mul(else_bb, Type::I32, acc, n).1;

    let call_res = f
        .make_call(else_bb, Type::I32, fact_id, vec![n_minus_1, acc_mul_n])
        .1;

    f.make_ret(else_bb, Some(call_res));

    f
}

fn example1_def() -> FunctionDef {
    let mut f = FunctionDef::new();
    let entry = f.entry;

    let then_bb = f.new_block();
    let else_bb = f.new_block();

    let zero = f.make_iconst(entry, Type::I32, 0);
    let one = f.make_iconst(entry, Type::I32, 1);

    let cond = f.make_cmp(entry, CmpKind::Lt, zero, one).1;
    f.make_jumpif(entry, cond, then_bb, vec![], else_bb, vec![]);

    f.make_ret(then_bb, Some(one));
    f.make_ret(else_bb, Some(zero));

    f
}

fn main() {
    pretty_env_logger::init();
    let mut m = Module::new("mirssa");
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
    let mfib = m.declare_function(
        "fib",
        FunctionSignature::new(vec![Type::I32], Type::I32),
        Linkage::default(),
        CallingConvention::default(),
    );
    let mfact_tr = m.declare_function(
        "fact_tr",
        FunctionSignature::new(vec![Type::I32, Type::I32], Type::I32),
        Linkage::default(),
        CallingConvention::default(),
    );
    let mexample1 = m.declare_function(
        "example1",
        FunctionSignature::new(vec![], Type::I32),
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
    m.define_function(mfib, fib_def(mfib))
        .expect("define_function error");
    m.define_function(mfact_tr, fact_tr_def(mfact_tr))
        .expect("define_function error");
    m.define_function(mexample1, example1_def())
        .expect("define_function error");
    m.verify().expect("pre-opt verify error");
    fs::write("preopt-mirssa.dot", m.dump_dot()).expect("fs::write error");
    fs::write("preopt-mirssa.mir", m.dump()).expect("fs::write error");
    m.optimize();
    m.verify().expect("post-opt verify error");
    fs::write("mirssa.dot", m.dump_dot()).expect("fs::write error");
    fs::write("mirssa.mir", m.dump()).expect("fs::write error");
    let llvm_ctx = Context::create();
    let mut lowerer = LlvmLowerer::new(&llvm_ctx, "mirssa");
    lowerer.lower_module(&m);
    let llvm_module = lowerer.get_module();
    fs::write("mirssa.ll", llvm_module.print_to_string().to_str().unwrap())
        .expect("fs::write error");
}
