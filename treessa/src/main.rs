use fear::types::Type;
use treessa::{passes::PassManager, *};

fn main() {
    pretty_env_logger::init();

    let mut f = FunctionDef::new();

    let b0 = f.new_block();
    let x = f.add_block_param(b0, Type::I32);
    let c10 = f.append_expr(b0, Type::I32, Expr::Const(10));
    let tmp = f.append_expr(
        b0,
        Type::I32,
        Expr::Mul(Box::new(Expr::Var(x)), Box::new(Expr::Var(c10))),
    );
    let add = f.append_expr(
        b0,
        Type::I32,
        Expr::Add(Box::new(Expr::Var(tmp)), Box::new(Expr::Var(c10))),
    );
    let c2 = f.append_expr(b0, Type::I32, Expr::Const(2));
    let sub = f.append_expr(
        b0,
        Type::I32,
        Expr::Sub(Box::new(Expr::Var(add)), Box::new(Expr::Var(c2))),
    );
    let sub2 = f.append_expr(
        b0,
        Type::I32,
        Expr::Sub(Box::new(Expr::Var(add)), Box::new(Expr::Var(c2))),
    );
    let mul = f.append_expr(
        b0,
        Type::I32,
        Expr::Mul(Box::new(Expr::Var(sub)), Box::new(Expr::Var(sub2))),
    );
    f.blocks.get_mut(&b0).unwrap().terminator = Some(Terminator::Ret(mul));

    println!("{}", f.dump());

    let res = PassManager::run_function(&mut f);
    println!("passes: {:?}", res.passes);

    println!("{}", f.dump())
}
