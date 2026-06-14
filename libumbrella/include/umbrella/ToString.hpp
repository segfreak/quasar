#pragma once

#include <string_view>

#include "Instruction.hpp"
#include "Type.hpp"

namespace umbrella
{

std::string_view toString(Opcode opcode);
std::string_view toString(OperandRole role);
std::string_view toString(TypeKind kind);
std::string_view toString(Type type);
std::string      toString(const Instruction& instr);
std::string      toString(VirtualRegister reg);

}  // namespace umbrella
