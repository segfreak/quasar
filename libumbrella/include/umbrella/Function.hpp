#pragma once

#include <cstddef>
#include <map>
#include <span>
#include <vector>

#include "BasicBlock.hpp"
#include "umbrella/VirtualRegister.hpp"

namespace umbrella {

struct Function
{
    Function(std::vector<BasicBlock> blocks) : blocks_(std::move(blocks))
    {
    }

    // get blocks view
    const std::vector<BasicBlock>& getBlocks() const { return blocks_; }
    // get mutable blocks
    std::vector<BasicBlock>&       getBlocks() { return blocks_; }
    bool isEmpty() const { return blocks_.empty(); }

    // get arguments view
    std::span<const VirtualRegister> getArguments() const
    { return arguments_; }
    // get mutable arguments
    std::vector<VirtualRegister>& getArguments() { return arguments_; }

   private:
    std::vector<VirtualRegister> arguments_;
    std::vector<BasicBlock>      blocks_;
};

}  // namespace umbrella
