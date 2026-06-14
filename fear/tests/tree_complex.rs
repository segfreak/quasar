use std::fs::File;
use std::io::BufWriter;
use std::path::PathBuf;

use fear::compiler::CompilerConfig;
use fear::tree::passes::PassManager;
use fear::tree::passes::Pipeline;
use fear::tree::*;
use fear::types::*;

#[allow(unused)]
fn test_def() -> FunctionDef {
    let mut f = FunctionDef::new();
    let entry = f.get_entry();

    // entry params
    let x = f.add_block_param(entry, Type::Int32);
    let y = f.add_block_param(entry, Type::Int32);

    // constants
    let c1 = f.make_iconst(entry, Type::Int32, 9);
    let c2 = f.make_iconst(entry, Type::Int32, 7);
    let c3 = f.make_iconst(entry, Type::Int32, 12);
    let c4 = f.make_iconst(entry, Type::Int32, 3);

    let fc1 = f.make_fconst(entry, Type::Float64, 6.62_f64.to_bits());
    let fc2 = f.make_fconst(entry, Type::Float64, 3.48_f64.to_bits());

    // arithmetic + redundancy
    let t1 = f.make_mul(entry, Type::Int32, &x, &c1);
    let t2 = f.make_add(entry, Type::Int32, &t1, &c2);
    let sq = f.make_square(entry, Type::Int32, &t2);

    // bit operations
    let shl = f.make_shl(entry, Type::Int32, &x, &c3);
    let band = f.make_bitand(entry, Type::Int32, &sq, &shl);

    // second redundant square (CSE target)
    let sq2 = f.make_square(entry, Type::Int32, &t2);

    let mix = f.make_add(entry, Type::Int32, &band, &sq2);

    // float ops
    let fadd = f.make_fadd(entry, Type::Float64, &fc1, &fc2);
    let fmul = f.make_fmul(entry, Type::Float64, &fc1, &fc2);

    // casts (both directions)
    let i64v = f.make_cast(entry, Type::Int64, CastKind::Sext, &mix);
    let ftoi = f.make_cast(entry, Type::Int32, CastKind::FPToSI, &fadd);
    let itof = f.make_cast(entry, Type::Float64, CastKind::SIToFP, &x);

    // memory
    let ptr = f.make_alloca(entry, Type::Int32);
    let _store = f.make_store(entry, false, &ptr, &mix);
    let load = f.make_load(entry, Type::Int32, false, &ptr);

    // pointer arithmetic
    let off = f.make_ptr_offset(entry, &ptr, &c4);
    let _gep = f.make_element_ptr(entry, Type::Int32, &off, &c2);

    // compare
    let cmp = f.make_icmp(entry, IntCmp::Eq, &x, &y);
    let fcmp = f.make_fcmp(entry, FloatCmp::OLt, &fc1, &fc2);

    // div/rem
    let div = f.make_div(entry, Type::Int32, false, &x, &c2);
    let rem = f.make_rem(entry, Type::Int32, false, &x, &c2);

    // unary bit ops
    let neg = f.make_bitneg(entry, Type::Int32, &x);

    // combine everything
    let load64 = f.make_cast(entry, Type::Int64, CastKind::Sext, &load);
    let sum1 = f.make_add(entry, Type::Int64, &i64v, &load64);
    let div64 = f.make_cast(entry, Type::Int64, CastKind::Sext, &div);
    let sum2 = f.make_add(entry, Type::Int64, &sum1, &div64);
    let rem64 = f.make_cast(entry, Type::Int64, CastKind::Sext, &rem);
    let sum3 = f.make_add(entry, Type::Int64, &sum2, &rem64);
    let neg64 = f.make_cast(entry, Type::Int64, CastKind::Sext, &neg);
    let sum4 = f.make_add(entry, Type::Int64, &sum3, &neg64);

    let finalv = f.make_cast(entry, Type::Int64, CastKind::Zext, &sum4);

    f.make_ret(entry, &finalv);

    f
}

#[test]
pub fn complex() {
    let _ = pretty_env_logger::try_init();

    let mut f = test_def();

    log::debug!("before opts:\n{}", f.dump());

    let pipeline = Pipeline::with_level(128, OptLevel::Default);

    let mut m = fear::ssa::Module::new("treessa");
    let res = PassManager::optimize_with_pipeline(&pipeline, &m, &mut f);
    println!("passes: {:?}", res.passes);

    println!("{}", f.dump());

    {
        let id = m.declare_function(
            "faz",
            FunctionSignature::new(vec![Type::Int32, Type::Int32], Type::Int64),
            fear::types::Linkage::External,
            fear::types::CallingConvention::C,
        );
        // defining a tree-ssa function (lowers into ssa)
        m.define_function(id, f).expect("cannot define function");
        m.optimize(OptLevel::Default, true);
        log::debug!("{}\n", m.dump());
        m.verify().expect("verify error");
        println!("{}", m.dump());

        fear::binary::write_to_file(&m, &PathBuf::from("treessac.bin"))
            .expect("cannot write fear binary module");
        std::fs::write("treessac.ssa", m.dump()).expect("cannot write fear text module");
    }

    {
        let config = CompilerConfig::setup(
            fear::compiler::OutputType::Object,
            target_lexicon::Triple::host(),
            OptLevel::Full,
        );
        let file = File::create("faz.o").unwrap();
        let writer = BufWriter::new(file);
        fear::compiler::compile_module(&m, &config, writer).expect("cannot compile module");
    }
}
