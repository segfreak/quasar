use std::{
    fs::{self, File},
    io::BufWriter,
    process::Command,
};

use fear::{binary, compiler::CompilerConfig, ssa::*, types::*};

fn foo_def() -> FunctionDef {
    let mut fun = FunctionDef::new();
    let b0 = fun.entry;
    let a0 = fun.add_param(Type::Int32);
    let a1 = fun.add_param(Type::Int32);
    let v0 = fun.make_iconst(b0, Type::Int32, 42);
    let v1 = fun.make_iconst(b0, Type::Int32, 2);
    let v2 = fun.make_mul(b0, fun.get_type(v0), v0, v1);
    let v3 = fun.make_div(b0, true, fun.get_type(v2), v2, a0);
    let v4 = fun.make_sub(b0, fun.get_type(v2), v3, a0);
    let v5 = fun.make_sub(b0, fun.get_type(v2), v4, a1);
    fun.make_ret(b0, Some(v5));
    fun
}

fn bar_def(mfoo: FuncId) -> FunctionDef {
    let mut fun = FunctionDef::new();
    let _a0 = fun.add_param(Type::Int32);
    let b0 = fun.entry;
    let v0 = fun.make_iconst(b0, Type::Int32, 42);
    let v1 = fun.make_iconst(b0, Type::Int32, 2);
    let v2 = fun.make_mul(b0, fun.get_type(v0), v0, v1);
    let v3 = fun.make_div(b0, true, fun.get_type(v2), v2, v0);
    let b1 = fun.new_block();
    fun.make_jump(b0, b1, vec![v3]);
    let v4 = fun.add_block_param(b1, Type::Int32);
    let v5 = fun.make_iconst(b1, Type::Int32, 2);
    let v6 = fun.make_call(b1, Type::Int32, mfoo, vec![v4, v5]);
    let v7 = fun.make_mul(b1, Type::Int32, v5, v6);
    fun.make_ret(b1, Some(v7));

    fun
}

fn baz_def() -> FunctionDef {
    let mut fun = FunctionDef::new();
    let b0 = fun.entry;
    let v0 = fun.make_alloca(b0, Type::Int32);
    fun.make_alloca(b0, Type::Int32);

    let v2 = fun.make_iconst(b0, Type::Int32, 42);
    let v3 = fun.make_iconst(b0, Type::Int32, 42 * 2);

    fun.make_store(b0, false, v0, v2);
    fun.make_store(b0, false, v0, v3);
    fun.make_ret(b0, None);
    fun
}

fn opt_def() -> FunctionDef {
    let mut fdef = FunctionDef::new();

    let entry = fdef.entry;

    let x = fdef.add_param(Type::Int32);

    // x * 2
    let two = fdef.make_iconst(entry, Type::Int32, 2);
    let x2 = fdef.make_mul(entry, Type::Int32, x, two);

    // (x * 2) + 0  -> dead + folding target
    let zero = fdef.make_iconst(entry, Type::Int32, 0);
    let x2_plus0 = fdef.make_add(entry, Type::Int32, x2, zero);

    // condition: (x * 2 + 0) > 10
    let ten = fdef.make_iconst(entry, Type::Int32, 10);
    let cond = fdef.make_cmp(entry, IntCmp::Gt, x2_plus0, ten);

    let then_bb = fdef.new_block();
    let else_bb = fdef.new_block();

    // THEN: (x * 2) + (x * 2)
    let then_x2_a = fdef.make_mul(then_bb, Type::Int32, x, two);
    let then_x2_b = fdef.make_mul(then_bb, Type::Int32, x, two);
    let then_res = fdef.make_add(then_bb, Type::Int32, then_x2_a, then_x2_b);
    fdef.make_ret(then_bb, Some(then_res));

    // ELSE: x * 2
    let else_x2 = fdef.make_mul(else_bb, Type::Int32, x, two);
    fdef.make_ret(else_bb, Some(else_x2));

    // entry jump
    fdef.make_jumpif(entry, cond, then_bb, vec![], else_bb, vec![]);

    fdef
}

fn fib_def(fib_id: FuncId) -> FunctionDef {
    let mut f = FunctionDef::new();
    let entry = f.entry;

    // param n
    let n = f.add_param(Type::Int32);

    // const 1
    let one = f.make_iconst(entry, Type::Int32, 1);

    // n <= 1
    let cond = f.make_cmp(entry, IntCmp::Le, n, one);

    let then_bb = f.new_block();
    let else_bb = f.new_block();

    // if (n <= 1) goto then else else
    f.make_jumpif(entry, cond, then_bb, vec![], else_bb, vec![]);

    // THEN: return n
    f.make_ret(then_bb, Some(n));

    // ELSE:
    // n - 1
    let n_minus_1 = f.make_sub(else_bb, Type::Int32, n, one);
    let call_fib_1 = f.make_call(else_bb, Type::Int32, fib_id, vec![n_minus_1]);

    // n - 2
    let two = f.make_iconst(else_bb, Type::Int32, 2);
    let n_minus_2 = f.make_sub(else_bb, Type::Int32, n, two);
    let call_fib_2 = f.make_call(else_bb, Type::Int32, fib_id, vec![n_minus_2]);

    // fib(n-1) + fib(n-2)
    let sum = f.make_add(else_bb, Type::Int32, call_fib_1, call_fib_2);

    f.make_ret(else_bb, Some(sum));

    f
}

// tail-recursive factorial
fn fact_tr_def(fact_id: FuncId) -> FunctionDef {
    let mut f = FunctionDef::new();
    let entry = f.entry;

    let n = f.add_param(Type::Int32);
    let acc = f.add_param(Type::Int32);

    let one = f.make_iconst(entry, Type::Int32, 1);

    let cond = f.make_cmp(entry, IntCmp::Le, n, one);

    let then_bb = f.new_block();
    let else_bb = f.new_block();

    f.make_jumpif(entry, cond, then_bb, vec![], else_bb, vec![]);

    f.make_ret(then_bb, Some(acc));

    let n_minus_1 = f.make_sub(else_bb, Type::Int32, n, one);

    let acc_mul_n = f.make_mul(else_bb, Type::Int32, acc, n);

    let call_res = f.make_call(else_bb, Type::Int32, fact_id, vec![n_minus_1, acc_mul_n]);

    f.make_ret(else_bb, Some(call_res));

    f
}

fn example1_def() -> FunctionDef {
    let mut f = FunctionDef::new();
    let entry = f.entry;

    let then_bb = f.new_block();
    let else_bb = f.new_block();

    let zero = f.make_iconst(entry, Type::Int32, 0);
    let one = f.make_iconst(entry, Type::Int32, 1);

    let cond = f.make_cmp(entry, IntCmp::Lt, zero, one);
    f.make_jumpif(entry, cond, then_bb, vec![], else_bb, vec![]);

    f.make_ret(then_bb, Some(one));
    f.make_ret(else_bb, Some(zero));

    f
}

#[test]
fn test() {
    pretty_env_logger::try_init().expect("cannot initialize logging");

    let mut m = Module::new("fear");
    let mfoo = m.declare_function(
        "foo",
        FunctionSignature::new(vec![Type::Int32, Type::Int32], Type::Int32),
        Linkage::default(),
        CallingConvention::default(),
    );
    let mbar = m.declare_function(
        "bar",
        FunctionSignature::new(vec![Type::Int32], Type::Int32),
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
        FunctionSignature::new(vec![Type::Int32], Type::Int32),
        Linkage::default(),
        CallingConvention::default(),
    );
    let mfib = m.declare_function(
        "fib",
        FunctionSignature::new(vec![Type::Int32], Type::Int32),
        Linkage::default(),
        CallingConvention::default(),
    );
    let mfact_tr = m.declare_function(
        "fact_tr",
        FunctionSignature::new(vec![Type::Int32, Type::Int32], Type::Int32),
        Linkage::default(),
        CallingConvention::default(),
    );
    let mexample1 = m.declare_function(
        "example1",
        FunctionSignature::new(vec![], Type::Int32),
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
    fs::write("preopt-fear.dot", m.dump_dot()).expect("fs::write error");
    fs::write("preopt-fear.ssa", m.dump()).expect("fs::write error");
    m.optimize(OptLevel::Default, false);
    // m.verify().expect("post-opt verify error");
    fs::write("fear.dot", m.dump_dot()).expect("fs::write error");
    fs::write("fear.ssa", m.dump()).expect("fs::write error");
    let file = File::create("fear.bin").unwrap();
    let writer = BufWriter::new(file);
    binary::write(&m, writer).unwrap();

    {
        let config = CompilerConfig::setup(
            fear::compiler::OutputType::Object,
            target_lexicon::Triple::host(),
            OptLevel::None,
        );
        let file = File::create("fear.o").unwrap();
        let writer = BufWriter::new(file);
        fear::compiler::compile_module(&m, &config, writer).expect("cannot compile module");
        let status = Command::new("cc")
            .arg("tests/test-ssa.c")
            .arg("fear.o")
            .arg("-o")
            .arg("test-ssa.x")
            .status()
            .expect("cannot run cc");
        if !status.success() {
            panic!("unsuccessfully status");
        }
        let status = Command::new("./test-ssa.x")
            .status()
            .expect("cannot run tests");
        if !status.success() {
            panic!("tests failed");
        }
    }

    // #[cfg(feature = "llvm")]
    // {
    //     use fear::ssa::lowering::llvm::LlvmLowerer;
    //     use inkwell::context::Context;
    //     use target_lexicon::Triple;

    //     let llvm_ctx = Context::create();
    //     let mut lowerer = LlvmLowerer::new(&m.name, Triple::host(), &llvm_ctx);
    //     lowerer.lower_module(&m);
    //     let llvm_module = lowerer.get_module();
    //     fs::write("fear.ll", llvm_module.print_to_string().to_str().unwrap())
    //         .expect("fs::write error");
    // }

    // #[cfg(feature = "cranelift")]
    // {
    //     use cranelift::codegen::{isa, settings::Configurable};
    //     use cranelift_module::default_libcall_names;
    //     use fear::ssa::lowering::cranelift::CraneliftLowerer;
    //     use target_lexicon::Triple;

    //     let mut flag_builder = cranelift::codegen::settings::builder();
    //     flag_builder.set("use_colocated_libcalls", "false").unwrap();
    //     flag_builder.set("is_pic", "true").unwrap();
    //     let flags = cranelift::codegen::settings::Flags::new(flag_builder);

    //     let isa = isa::lookup(Triple::host()).unwrap().finish(flags).unwrap();

    //     let mut lowerer = CraneliftLowerer::new(&m.name, isa, default_libcall_names());
    //     lowerer.lower_module(&m);
    //     let object_bytes = lowerer.finish();
    //     fs::write("fear.o", object_bytes).expect("fs::write error");
    // }
}
