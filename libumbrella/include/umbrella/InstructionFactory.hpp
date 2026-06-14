#pragma once

#include "Instruction.hpp"
#include "VirtualRegister.hpp"

namespace umbrella
{

struct InstructionFactory
{
    static Instruction createAdd(VirtualRegister dst, VirtualRegister src,
                                 VirtualRegister src2)
    {
        return {
            Opcode::Add, std::vector{Operand{dst, OperandRole::Dst},
                                     Operand{src, OperandRole::Src},
                                     Operand{src2, OperandRole::Src}}
        };
    }

    static Instruction createSub(VirtualRegister dst, VirtualRegister src,
                                 VirtualRegister src2)
    {
        return {
            Opcode::Sub, std::vector{Operand{dst, OperandRole::Dst},
                                     Operand{src, OperandRole::Src},
                                     Operand{src2, OperandRole::Src}}
        };
    }

    static Instruction createRet(VirtualRegister src)
    { return {Opcode::Ret, std::vector{Operand{src, OperandRole::Src}}}; }
};

}  // namespace umbrella
