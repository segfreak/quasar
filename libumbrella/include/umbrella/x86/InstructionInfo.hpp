#pragma once

#include "InstructionSet.hpp"
#include "Operand.hpp"

namespace umbrella::x86
{

struct InstructionInfo
{
    InstructionInfo(Opcode opcode) : opcode_(opcode) {}

    static InstructionInfo get(Opcode opcode)
    { return InstructionInfo{opcode}; }

    std::span<const OperandKind> getExplicitOperandKinds();
    std::vector<Operand>         getImplicitOperands();

    Opcode                       getOpcode() const { return opcode_; }

   private:
    Opcode opcode_;
};

}  // namespace umbrella::x86