use std::fs;

use quasar::{target::CallingConvention, *};
use ril::{
    ir::*,
    live,
    regalloc::{PhysReg, RegAlloc},
};

fn foo_def() -> FunctionDef {
    let mut fun = FunctionDef::new();
    let b0 = fun.entry;

    fun.make_add(
        b0,
        VReg::new(0, Type::I32),
        Operand::imm(20),
        Operand::imm(22),
    );

    fun.make_add(
        b0,
        VReg::new(1, Type::I32),
        Operand::reg(0, Type::I32),
        Operand::imm(22),
    );

    fun.make_add(
        b0,
        VReg::new(2, Type::I32),
        Operand::reg(1, Type::I32),
        Operand::imm(2),
    );

    fun.make_add(
        b0,
        VReg::new(3, Type::I32),
        Operand::reg(2, Type::I32),
        Operand::imm(2),
    );

    fun.make_add(
        b0,
        VReg::new(4, Type::I32),
        Operand::reg(2, Type::I32),
        Operand::imm(2),
    );

    fun.make_add(
        b0,
        VReg::new(5, Type::I32),
        Operand::reg(2, Type::I32),
        Operand::imm(2),
    );

    fun.make_add(
        b0,
        VReg::new(6, Type::I32),
        Operand::reg(2, Type::I32),
        Operand::imm(2),
    );

    fun.make_add(
        b0,
        VReg::new(7, Type::I32),
        Operand::reg(2, Type::I32),
        Operand::imm(2),
    );

    fun.make_add(
        b0,
        VReg::new(8, Type::I32),
        Operand::reg(2, Type::I32),
        Operand::imm(2),
    );

    fun.make_ret(b0, Some(Operand::reg(0, Type::I32)));

    let live_intervals = live::build_live_intervals(&fun);
    let ra = RegAlloc::new(
        live_intervals,
        vec![
            PhysReg::new(0, Type::I32),
            PhysReg::new(1, Type::I32),
            PhysReg::new(2, Type::I32),
        ],
    );
    let res = ra.run();
    log::debug!("regalloc result: {}", res);

    fun
}

fn main() {
    pretty_env_logger::init();
    let mut m = Module::new("ril");
    let mfoo = m.declare_function(
        "foo",
        FunctionSignature::new(vec![Type::I32, Type::I32], Type::I32),
        Linkage::default(),
        CallingConvention::default(),
    );
    m.define_function(mfoo, foo_def())
        .expect("define_function error");
    fs::write("ril.dot", m.dump_dot()).expect("fs::write error");
}
