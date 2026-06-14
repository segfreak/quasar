#pragma once

#include <span>
#include <vector>

#include "InstructionSet.hpp"
#include "Operand.hpp"

namespace umbrella::x86 {

struct InstructionInfo
{
    InstructionInfo(X86Opcode opcode) : opcode_(opcode) {}

    static InstructionInfo get(X86Opcode opcode)
    { return InstructionInfo{opcode}; }

    std::span<const OperandKind> getExplicitOperandKinds() const;
    std::vector<Operand>         getImplicitOperands() const;

    X86Opcode                    getOpcode() const { return opcode_; }

   private:
    X86Opcode opcode_;
};

}  // namespace umbrella::x86