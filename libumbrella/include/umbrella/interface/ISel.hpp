#pragma once

#include <vector>

#include "../Instruction.hpp"

namespace umbrella
{

template <typename MachInstrT>
struct ISel
{
    virtual std::vector<MachInstrT> select(
        const std::vector<Instruction>& src) = 0;
    virtual ~ISel()                          = default;
};

}  // namespace umbrella