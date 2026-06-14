#pragma once

#include "../interface/ISel.hpp"
#include "Instruction.hpp"

namespace umbrella::x86 {

struct X86ISel : public umbrella::ISel<Instruction>
{
    std::vector<umbrella::x86::Instruction> select(
        const std::vector<umbrella::Instruction>& src) override;
};

}  // namespace umbrella::x86