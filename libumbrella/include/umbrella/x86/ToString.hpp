#pragma once

#include <string_view>

#include "Instruction.hpp"
#include "InstructionSet.hpp"
#include "Operand.hpp"
#include "Register.hpp"

namespace umbrella::x86 {

std::string_view toString(RegisterKind k);
std::string      toString(Register r);
std::string_view toString(X86Opcode opcode);
std::string_view toString(X86Mnemonic mnemonic);
std::string_view toString(OperandKind k);
std::string      toString(const Memory& mem);
std::string      toString(const Operand& op);
std::string      toString(const Instruction& instr);

}  // namespace umbrella::x86