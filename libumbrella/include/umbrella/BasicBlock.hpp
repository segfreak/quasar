#pragma once

#include "Instruction.hpp"

namespace umbrella
{

struct BasicBlock
{
    BasicBlock(std::vector<Instruction> instructions)
        : instructions_(std::move(instructions))
    {
    }

    const std::vector<Instruction>& getInstructions() const
    { return instructions_; }

    std::vector<Instruction>& getInstructions() { return instructions_; }

   private:
    std::vector<Instruction> instructions_;
};

}  // namespace umbrella
