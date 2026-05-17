use std::fs;

use quasar::{target::CallingConvention, *};
use ril::{
    interval,
    ir::*,
    regalloc::{Reg, RegAlloc, RegClass},
    regalloc2::RegAlloc2,
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

    fun
}

fn demonstrate_regalloc(fun: &FunctionDef) {
    let live_intervals = interval::build_live_intervals(fun);
    log::info!("intervals: {:?}", live_intervals);

    let phys_regs = vec![
        Reg::new(0, Type::I32, RegClass::General),
        Reg::new(1, Type::I32, RegClass::General),
        Reg::new(2, Type::I32, RegClass::General),
    ];

    let mut ra_standard = RegAlloc::new(live_intervals.clone(), phys_regs.clone());
    let result_standard = ra_standard.linear_scan();
    log::info!("Standard linear scan result: {}", result_standard);

    let mut ra_improved = RegAlloc2::new(live_intervals, phys_regs);
    let result_improved = ra_improved.linear_scan();
    log::info!("Improved linear scan result: {}", result_improved);
    log::info!(
        "Stack frame size: {} bytes",
        ra_improved.get_stack_frame_size()
    );
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
    let fun = foo_def();
    demonstrate_regalloc(&fun);
    // let mut fun_copy = fun.clone();
    // demonstrate_coalescing(&mut fun_copy);
    m.define_function(mfoo, fun).expect("define_function error");

    fs::write("ril.dot", m.dump_dot()).expect("fs::write error");
}
