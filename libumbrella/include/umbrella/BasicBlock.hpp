#pragma once

#include <cstddef>
#include <map>
#include <vector>

#include "Instruction.hpp"

namespace umbrella {

struct BasicBlock
{
    BasicBlock(std::vector<Instruction> instructions)
        : instructions_(std::move(instructions))
    {
    }

    const std::vector<Instruction>& getInstructions() const
    { return instructions_; }

    std::vector<Instruction>& getInstructions() { return instructions_; }

    bool isEmpty() const { return instructions_.empty(); }

    void addInstruction(const Instruction& instruction)
    { instructions_.push_back(instruction); }

    std::map</* instruction index*/ std::size_t, VirtualRegister>
    getRegistersUsed() const
    {
        std::map</* instruction index*/ std::size_t, VirtualRegister>
            usedRegisters;
        for (const auto& instruction : instructions_) {
            for (std::size_t i = 0; i < instruction.getOperands().size();
                 ++i) {
                const auto& o = instruction.getOperands()[i];
                if (o.isSource() && o.isRegister()) {
                    usedRegisters[i] = (o.getRegister().value());
                }
            }
        }
        return usedRegisters;
    }

    std::map</* instruction index*/ std::size_t, VirtualRegister>
    getRegistersDefined() const
    {
        std::map</* instruction index*/ std::size_t, VirtualRegister>
            usedRegisters;
        for (const auto& instruction : instructions_) {
            for (std::size_t i = 0; i < instruction.getOperands().size();
                 ++i) {
                const auto& o = instruction.getOperands()[i];
                if (o.isDestination() && o.isRegister()) {
                    usedRegisters[i] = (o.getRegister().value());
                }
            }
        }
        return usedRegisters;
    }

   private:
    std::vector<Instruction> instructions_;
};

}  // namespace umbrella
