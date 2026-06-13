#pragma once

#include "Instruction.hpp"
#include "Register.hpp"

namespace umbrella
{

struct InstructionFactory
{
    static Instruction createAdd(Register dst, Register src, Register src2)
    {
        return {
            Opcode::Add, std::vector{Operand{dst, OperandRole::Dst},
                                     Operand{src, OperandRole::Src},
                                     Operand{src2, OperandRole::Src}}
        };
    }

    static Instruction createSub(Register dst, Register src, Register src2)
    {
        return {
            Opcode::Sub, std::vector{Operand{dst, OperandRole::Dst},
                                     Operand{src, OperandRole::Src},
                                     Operand{src2, OperandRole::Src}}
        };
    }

    static Instruction createRet(Register src)
    { return {Opcode::Ret, std::vector{Operand{src, OperandRole::Src}}}; }
};

}  // namespace umbrella
