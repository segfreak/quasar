#pragma once

#include <string_view>

#include "InstructionSet.hpp"
#include "Operand.hpp"
#include "Register.hpp"

namespace umbrella::x86
{

std::string_view toString(RegisterKind k);
std::string_view toString(Register reg);
std::string_view toString(Opcode opcode);
std::string_view toString(OperandKind k);
std::string      toString(const Memory& mem);
std::string      toString(const Operand& op);

}  // namespace umbrella::x86