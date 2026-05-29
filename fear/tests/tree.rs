use std::{fs::File, io::BufWriter};

use fear::types::{CastKind, FunctionSignature, OptLevel, Type};
use fear::{
    compiler::CompilerConfig,
    tree::{
        passes::{PassManager, Pipeline},
        *,
    },
};

#[allow(unused)]
#[test]
fn test() {
    pretty_env_logger::init();

    let mut f = FunctionDef::new();

    let b0 = f.get_entry();
    let x0 = f.add_block_param(b0, Type::Int32);

    let slot0 = f.make_alloca(b0, Type::Int32);
    let c228 = f.make_iconst(b0, Type::Int32, 228);
    f.make_store(b0, false, &slot0, &c228);

    let b1 = f.new_block();
    let x = f.add_block_param(b1, Type::Int32);

    f.make_br(b0, b1, vec![x0]);

    let c9 = f.make_iconst(b1, Type::Int32, 9);
    let tmp = f.make_mul(b1, Type::Int32, &x, &c9);
    let add = f.make_add(b1, Type::Int32, &tmp, &c9);
    let c2 = f.make_iconst(b1, Type::Int32, 2);
    let c4 = f.make_iconst(b1, Type::Int32, 4);
    let c64 = f.make_iconst(b1, Type::Int32, 64);

    let sub = f.make_sub(b1, Type::Int32, &add, &c2);
    let sub2 = f.make_sub(b1, Type::Int32, &add, &c2);
    let mul = f.make_mul(b1, Type::Int32, &sub, &sub2);
    let div = f.make_div(b1, Type::Int32, true, &mul, &c2);
    let mul2 = f.make_mul(b1, Type::Int32, &div, &c2);
    let mul64 = f.make_mul(b1, Type::Int32, &x, &c64);
    let div64 = f.make_mul(b1, Type::Int32, &mul64, &c64);
    let sum = f.make_add(b1, Type::Int32, &mul2, &div64);
    let nonprofit = f.make_mul(
        b1,
        Type::Int32,
        &sum,
        &Expr {
            ty: Type::Int32,
            kind: ExprKind::Const(0x1000000),
        },
    );
    let square = f.make_mul(b1, Type::Int32, &x, &x);
    let mul8 = f.make_mul(
        b1,
        Type::Int32,
        &square,
        &Expr {
            ty: Type::Int32,
            kind: ExprKind::Const(8),
        },
    );
    let y = f.make_add(b1, Type::Int32, &nonprofit, &mul8);
    let decompose = f.make_mul(
        b1,
        Type::Int32,
        &sum,
        &Expr {
            ty: Type::Int32,
            kind: ExprKind::Const(9),
        },
    );
    let z = f.make_sub(b1, Type::Int32, &y, &decompose);
    let slot0v = f.make_load(b1, Type::Int32, false, &slot0);
    let x0 = f.make_add(b1, Type::Int32, &z, &slot0v);
    let x0_64 = f.make_cast(b1, Type::Int64, CastKind::Sext, &x0);

    let x = f.make_square(b1, Type::Int32, &x0);
    let _y = f.make_iconst(b1, Type::Int32, 42);
    let y = f.make_square(b1, Type::Int32, &_y);
    let z = f.make_sub(b1, Type::Int32, &x, &y);
    let z64 = f.make_cast(b1, Type::Int64, CastKind::Sext, &z);

    let ret = f.make_bitand(b1, Type::Int64, &x0_64, &z64);
    f.make_ret(b1, &ret);
    println!("{}", f.dump());

    log::debug!("before opts:\n{}", f.dump());

    let mut pipeline = Pipeline::with_level(128, OptLevel::Default);
    // pipeline.get_passes_mut().remove(&PassKind::ConstantFolding);

    let mut m = fear::ssa::Module::new("treessa");
    let res = PassManager::optimize_with_pipeline(&pipeline, &m, &mut f);
    println!("passes: {:?}", res.passes);

    println!("{}", f.dump());

    {
        let mfoo = m.declare_function(
            "foo",
            FunctionSignature::new(vec![Type::Int32], Type::Int64),
            fear::types::Linkage::External,
            fear::types::CallingConvention::C,
        );
        // defining a tree-ssa function (lowers into ssa)
        m.define_function(mfoo, f).expect("cannot define function");
        m.optimize(OptLevel::Default, true);
        log::debug!("{}\n", m.dump());
        m.verify().expect("verify error");
        println!("{}", m.dump());

        fear::binary::write_to_file(&m, "treessa.bin").expect("cannot write fear binary module");
        std::fs::write("treessa.ssa", m.dump()).expect("cannot write fear text module");
    }

    {
        let config = CompilerConfig::setup(
            fear::compiler::OutputType::Object,
            target_lexicon::Triple::host(),
            OptLevel::None,
        );
        let file = File::create("foo.o").unwrap();
        let writer = BufWriter::new(file);
        fear::compiler::compile_module(&m, &config, writer).expect("cannot compile module");
    }
}
