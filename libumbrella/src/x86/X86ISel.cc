
#include "umbrella/x86/X86ISel.hpp"

#include "umbrella/Instruction.hpp"
#include "umbrella/ToString.hpp"
#include "umbrella/support/Logging.hpp"
#include "umbrella/x86/InstructionSet.hpp"
#include "umbrella/x86/Operand.hpp"

namespace umbrella::x86 {

bool selectAdd(std::vector<Instruction>& out, const umbrella::Operand& dst,
               const umbrella::Operand& src1,
               const umbrella::Operand& src2)
{
    // Destination must be a register
    if (!dst.isRegister()) {
        errs("X86ISel") << "add dst must be a register\n";
        return false;
    }

    auto dstReg = Register{dst.getRegister().value()};

    // Case 1: dst == src1, use: add dst, src2
    if (dst.isRegister() && src1.isRegister() &&
        dst.getRegister().value() == src1.getRegister().value()) {
        std::vector<Operand> operands;
        operands.push_back(Operand{dstReg, OperandRole::DstSrc});

        if (src2.isRegister()) {
            operands.push_back(Operand{
                Register{src2.getRegister().value()}, OperandRole::Src});
            out.emplace_back(X86Mnemonic::Add, operands);
        } else if (src2.isImmediate()) {
            operands.push_back(
                Operand{src2.getImmediate().value(), OperandRole::Src});
            out.emplace_back(X86Mnemonic::Add, operands);
        } else {
            errs("X86ISel") << "add src2 must be register or immediate\n";
            return false;
        }
        return true;
    }

    // Case 2: dst == src2, use: add dst, src1
    if (dst.isRegister() && src2.isRegister() &&
        dst.getRegister().value() == src2.getRegister().value()) {
        std::vector<Operand> operands;
        operands.push_back(Operand{dstReg, OperandRole::DstSrc});

        if (src1.isRegister()) {
            operands.push_back(Operand{
                Register{src1.getRegister().value()}, OperandRole::Src});
            out.emplace_back(X86Mnemonic::Add, operands);
        } else if (src1.isImmediate()) {
            operands.push_back(
                Operand{src1.getImmediate().value(), OperandRole::Src});
            out.emplace_back(X86Mnemonic::Add, operands);
        } else {
            errs("X86ISel") << "add src1 must be register or immediate\n";
            return false;
        }
        return true;
    }

    // Case 3: dst != src1 && dst != src2
    bool is8Bit =
        (dst.getRegister()->getType().getKind() == TypeKind::Int8);

    if (is8Bit) {
        std::vector<Operand> movOperands;
        movOperands.push_back(Operand{dstReg, OperandRole::Dst});

        if (src1.isRegister()) {
            movOperands.push_back(Operand{
                Register{src1.getRegister().value()}, OperandRole::Src});
        } else if (src1.isImmediate()) {
            movOperands.push_back(
                Operand{src1.getImmediate().value(), OperandRole::Src});
        } else {
            errs("X86ISel") << "8-bit fallback mov src1 must be register "
                               "or immediate\n";
            return false;
        }
        out.emplace_back(X86Mnemonic::Mov, movOperands);

        std::vector<Operand> addOperands;
        addOperands.push_back(Operand{dstReg, OperandRole::DstSrc});

        if (src2.isRegister()) {
            addOperands.push_back(Operand{
                Register{src2.getRegister().value()}, OperandRole::Src});
        } else if (src2.isImmediate()) {
            addOperands.push_back(
                Operand{src2.getImmediate().value(), OperandRole::Src});
        } else {
            errs("X86ISel") << "8-bit fallback add src2 must be register "
                               "or immediate\n";
            return false;
        }
        out.emplace_back(X86Mnemonic::Add, addOperands);

        return true;
    }

    if (!src1.isRegister() || !src2.isRegister()) {
        errs("X86ISel") << "lea requires both sources to be registers\n";
        return false;
    }

    auto                 src1Reg = Register{src1.getRegister().value()};
    auto                 src2Reg = Register{src2.getRegister().value()};

    Memory               mem{src1Reg, 0, src2Reg, Scale::One};

    std::vector<Operand> operands;
    operands.push_back(Operand{dstReg, OperandRole::Dst});
    operands.push_back(Operand{mem, OperandRole::Src});
    out.emplace_back(X86Mnemonic::Lea, operands);

    return true;
}

bool selectSub(std::vector<Instruction>& out, const umbrella::Operand& dst,
               const umbrella::Operand& src1,
               const umbrella::Operand& src2)
{
    // Destination must be a register
    if (!dst.isRegister()) {
        errs("X86ISel") << "sub dst must be a register\n";
        return false;
    }

    auto dstReg = Register{dst.getRegister().value()};

    // Case 1: dst == src1, use: sub dst, src2
    if (dst.isRegister() && src1.isRegister() &&
        dst.getRegister().value() == src1.getRegister().value()) {
        std::vector<Operand> operands;
        operands.push_back(Operand{dstReg, OperandRole::DstSrc});

        if (src2.isRegister()) {
            operands.push_back(Operand{
                Register{src2.getRegister().value()}, OperandRole::Src});
            out.emplace_back(X86Mnemonic::Sub, operands);
        } else if (src2.isImmediate()) {
            operands.push_back(
                Operand{src2.getImmediate().value(), OperandRole::Src});
            out.emplace_back(X86Mnemonic::Sub, operands);
        } else {
            errs("X86ISel") << "sub src2 must be register or immediate\n";
            return false;
        }
        return true;
    }

    // Case 2: dst == src2, need to load src1 first then subtract
    // Use: mov dst, src1; sub dst, src2
    if (dst.isRegister() && src2.isRegister() &&
        dst.getRegister().value() == src2.getRegister().value()) {
        // First: mov dst, src1
        std::vector<Operand> movOperands;
        movOperands.push_back(Operand{dstReg, OperandRole::Dst});

        if (src1.isRegister()) {
            movOperands.push_back(Operand{
                Register{src1.getRegister().value()}, OperandRole::Src});
            out.emplace_back(X86Mnemonic::Mov, movOperands);
        } else if (src1.isImmediate()) {
            movOperands.push_back(
                Operand{src1.getImmediate().value(), OperandRole::Src});
            out.emplace_back(X86Mnemonic::Mov, movOperands);
        } else {
            errs("X86ISel") << "sub src1 must be register or immediate\n";
            return false;
        }

        // Second: sub dst, src2
        std::vector<Operand> subOperands;
        subOperands.push_back(Operand{dstReg, OperandRole::DstSrc});
        subOperands.push_back(Operand{Register{src2.getRegister().value()},
                                      OperandRole::Src});
        out.emplace_back(X86Mnemonic::Sub, subOperands);
        return true;
    }

    // Case 3: dst != src1 && dst != src2
    // Use: mov dst, src1; sub dst, src2
    std::vector<Operand> movOperands;
    movOperands.push_back(Operand{dstReg, OperandRole::Dst});

    if (src1.isRegister()) {
        movOperands.push_back(Operand{Register{src1.getRegister().value()},
                                      OperandRole::Src});
        out.emplace_back(X86Mnemonic::Mov, movOperands);
    } else if (src1.isImmediate()) {
        movOperands.push_back(
            Operand{src1.getImmediate().value(), OperandRole::Src});
        out.emplace_back(X86Mnemonic::Mov, movOperands);
    } else {
        errs("X86ISel") << "sub src1 must be register or immediate\n";
        return false;
    }

    // Second: sub dst, src2
    std::vector<Operand> subOperands;
    subOperands.push_back(Operand{dstReg, OperandRole::DstSrc});

    if (src2.isRegister()) {
        subOperands.push_back(Operand{Register{src2.getRegister().value()},
                                      OperandRole::Src});
        out.emplace_back(X86Mnemonic::Sub, subOperands);
    } else if (src2.isImmediate()) {
        subOperands.push_back(
            Operand{src2.getImmediate().value(), OperandRole::Src});
        out.emplace_back(X86Mnemonic::Sub, subOperands);
    } else {
        errs("X86ISel") << "sub src2 must be register or immediate\n";
        return false;
    }

    return true;
}

std::vector<Instruction> X86ISel::select(
    const std::vector<umbrella::Instruction>& src)
{
    std::vector<Instruction> result;

    for (const auto& instr : src) {
        if (instr.verify()) {
            errs("X86ISel") << "instruction verify failure, skip..\n";
            errs("X86ISel") << "your code is bogus at this point\n";
            continue;
        }

        switch (instr.getOpcode()) {
            case Opcode::Add:
            {
                auto        operands = instr.getOperands();
                const auto& dst      = operands[0];
                const auto& src1     = operands[1];
                const auto& src2     = operands[2];
                if (!selectAdd(result, dst, src1, src2)) {
                    errs("X86ISel") << "add selection failed\n";
                }
                break;
            }
            case Opcode::Sub:
            {
                auto        operands = instr.getOperands();
                const auto& dst      = operands[0];
                const auto& src1     = operands[1];
                const auto& src2     = operands[2];
                if (!selectSub(result, dst, src1, src2)) {
                    errs("X86ISel") << "sub selection failed\n";
                }
                break;
            }

            default:
            {
                errs("X86ISel")
                    << "unhandled opcode: " << toString(instr.getOpcode())
                    << "\n";
                break;
            }
        }
    }

    return result;
}

}  // namespace umbrella::x86