use fear::tree::{passes::PassManager, *};
use fear::types::{FunctionSignature, OptLevel, Type};

#[test]
fn test() {
    pretty_env_logger::init();

    let mut f = FunctionDef::new();

    let b0 = f.get_entry();
    let x0 = f.add_block_param(b0, Type::I32);

    let slot0 = f.make_alloca(b0, Type::I32);
    let c228 = f.make_iconst(b0, Type::I32, 228);
    f.make_store(b0, false, slot0.into(), c228.into());

    let b1 = f.new_block();
    let x = f.add_block_param(b1, Type::I32);

    f.make_br(b0, b1, vec![x0]);

    let c9 = f.append_expr(b1, Type::I32, Expr::Const(9));
    let tmp = f.append_expr(
        b1,
        Type::I32,
        Expr::Mul(Box::new(Expr::Var(x)), Box::new(Expr::Var(c9))),
    );

    let add = f.make_add(b1, Type::I32, tmp.into(), c9.into());
    let c2 = f.make_iconst(b1, Type::I32, 2);
    let sub = f.make_sub(b1, Type::I32, add.into(), c2.into());
    let sub2 = f.append_expr(
        b1,
        Type::I32,
        Expr::Sub(Box::new(Expr::Var(add)), Box::new(Expr::Var(c2))),
    );
    let mul = f.make_mul(b1, Type::I32, sub.into(), sub2.into());
    let div = f.make_div(b1, Type::I32, true, mul.into(), Expr::Const(2));
    let mul2 = f.make_mul(b1, Type::I32, div.into(), Expr::Const(2));
    let mul64 = f.make_mul(b1, Type::I32, x.into(), Expr::Const(64));
    let div64 = f.make_mul(b1, Type::I32, mul64.into(), Expr::Const(64));
    let sum = f.make_add(b1, Type::I32, mul2.into(), div64.into());
    let nonprofit = f.make_mul(b1, Type::I32, sum.into(), Expr::Const(0x1000000));
    let square = f.make_mul(b1, Type::I32, x.into(), x.into());
    let mul8 = f.make_mul(b1, Type::I32, square.into(), Expr::Const(8));
    let y = f.make_add(b1, Type::I32, nonprofit.into(), mul8.into());
    let decompose = f.make_mul(b1, Type::I32, sum.into(), Expr::Const(9));
    let z = f.make_sub(b1, Type::I32, y.into(), decompose.into());
    let slot0v = f.make_load(b1, Type::I32, false, slot0.into());
    let c = f.make_add(b1, Type::I32, z.into(), slot0v.into());
    f.make_ret(b1, c);
    println!("{}", f.dump());

    let mut m = fear::ssa::Module::new("treessa");
    let res = PassManager::optimize(&m, &mut f, OptLevel::Default, i32::MAX);
    println!("passes: {:?}", res.passes);

    println!("{}", f.dump());

    {
        let mfoo = m.declare_function(
            "foo",
            FunctionSignature::new(vec![Type::I32], Type::I32),
            fear::types::Linkage::External,
            fear::types::CallingConvention::C,
        );
        // defining a tree function (lowers into ssa)
        m.define_function(mfoo, f).expect("cannot define function");
        m.optimize();
        m.verify().expect("verify error");
        println!("{}", m.dump());

        fear::binary::write_to_file(&m, "treessa.bin").expect("cannot write fear binary module");
        std::fs::write("treessa.ssa", m.dump()).expect("cannot write fear text module");
    }
}
