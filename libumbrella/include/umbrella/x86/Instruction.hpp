#pragma once

#include "InstructionInfo.hpp"
#include "InstructionSet.hpp"
#include "Operand.hpp"

namespace umbrella::x86
{

struct Instruction
{
    Instruction(Opcode opcode, std::vector<Operand> operands)
        : info_(opcode), operands_(std::move(operands))
    {
    }

    Opcode getOpcode() const { return getInfo().getOpcode(); }
    const InstructionInfo&      getInfo() const { return info_; }
    const std::vector<Operand>& getOperands() const { return operands_; }

   private:
    InstructionInfo      info_;
    std::vector<Operand> operands_;
};

}  // namespace umbrella::x86