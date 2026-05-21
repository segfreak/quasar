use fear::types::{OptLevel, Type};
use treessa::{passes::PassManager, *};

fn main() {
    pretty_env_logger::init();

    let mut f = FunctionDef::new();

    let b0 = f.get_entry();
    let x = f.add_block_param(b0, Type::I32);
    let c10 = f.append_expr(b0, Type::I32, Expr::Const(10));
    let tmp = f.append_expr(
        b0,
        Type::I32,
        Expr::Mul(Box::new(Expr::Var(x)), Box::new(Expr::Var(c10))),
    );

    let add = f.make_add(b0, Type::I32, tmp.into(), c10.into());
    let c2 = f.make_iconst(b0, Type::I32, 2);
    let sub = f.make_sub(b0, Type::I32, add.into(), c2.into());
    let sub2 = f.append_expr(
        b0,
        Type::I32,
        Expr::Sub(Box::new(Expr::Var(add)), Box::new(Expr::Var(c2))),
    );
    let mul = f.make_mul(b0, Type::I32, sub.into(), sub2.into());
    let div = f.make_div(b0, Type::I32, mul.into(), Expr::Const(2));
    let mul2 = f.make_mul(b0, Type::I32, div.into(), Expr::Const(2));
    let mul4 = f.make_mul(b0, Type::I32, div.into(), Expr::Const(4));
    let sum = f.make_add(b0, Type::I32, mul2.into(), mul4.into());
    let decompose = f.make_mul(
        b0,
        Type::I32,
        sum.into(),
        Expr::Const(0x3FFF_FFFF_FFFF_FFFF),
    );
    f.make_ret(b0, decompose);

    println!("{}", f.dump());

    let res = PassManager::optimize(&mut f, OptLevel::Default);
    println!("passes: {:?}", res.passes);

    println!("{}", f.dump())
}
