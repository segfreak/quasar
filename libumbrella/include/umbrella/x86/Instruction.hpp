#pragma once

#include "InstructionInfo.hpp"
#include "InstructionSet.hpp"
#include "OpcodeSel.hpp"
#include "Operand.hpp"

namespace umbrella::x86 {

struct Instruction
{
    Instruction(X86Mnemonic mnemonic, std::vector<Operand> operands)
        : mnemonic_(mnemonic),
          info_(OpcodeSel::select(mnemonic, operands)
                    .value_or(X86Opcode::Ud2)),
          operands_(std::move(operands))
    {
    }

    // Instruction(X86Opcode opcode, std::vector<Operand> operands)
    //     : info_(opcode), operands_(std::move(operands))
    // {
    // }

    X86Opcode   getOpcode() const { return getInfo().getOpcode(); }
    X86Mnemonic getMnemonic() const { return mnemonic_; }

    const InstructionInfo&      getInfo() const { return info_; }
    const std::vector<Operand>& getOperands() const { return operands_; }

   private:
    X86Mnemonic          mnemonic_{};
    InstructionInfo      info_;
    std::vector<Operand> operands_;
};

}  // namespace umbrella::x86