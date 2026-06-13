#pragma once

#include <ostream>

#include "InstructionSet.hpp"
#include "Register.hpp"

namespace umbrella::x86
{

std::ostream& operator<<(std::ostream& os, RegisterKind k);
std::ostream& operator<<(std::ostream& os, Register reg);
std::ostream& operator<<(std::ostream& os, Opcode op);

}  // namespace umbrella::x86