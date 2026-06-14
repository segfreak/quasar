
#include <iostream>
#include <umbrella/x86/ToString.hpp>

#include "umbrella/x86/InstructionSet.hpp"

namespace umbrella::x86
{

std::ostream& operator<<(std::ostream& os, RegisterKind k)
{ return os << toString(k); }

std::ostream& operator<<(std::ostream& os, Register reg)
{ return os << toString(reg); }

std::ostream& operator<<(std::ostream& os, X86Opcode opcode)
{ return os << toString(opcode); }

std::ostream& operator<<(std::ostream& os, OperandKind k)
{ return os << toString(k); }

std::ostream& operator<<(std::ostream& os, const Memory& mem)
{ return os << toString(mem); }

std::ostream& operator<<(std::ostream& os, const Operand& op)
{ return os << toString(op); }

std::ostream& operator<<(std::ostream& os, const Instruction& instr)
{ return os << toString(instr); }

}  // namespace umbrella::x86