#pragma once

#include <ostream>

#include "Instruction.hpp"
#include "InstructionSet.hpp"
#include "Operand.hpp"
#include "Register.hpp"

namespace umbrella::x86 {

std::ostream& operator<<(std::ostream& os, RegisterKind k);
std::ostream& operator<<(std::ostream& os, Register reg);
std::ostream& operator<<(std::ostream& os, X86Opcode op);
std::ostream& operator<<(std::ostream& os, OperandKind k);
std::ostream& operator<<(std::ostream& os, const Memory& mem);
std::ostream& operator<<(std::ostream& os, const Operand& op);
std::ostream& operator<<(std::ostream& os, const Instruction& instr);

}  // namespace umbrella::x86