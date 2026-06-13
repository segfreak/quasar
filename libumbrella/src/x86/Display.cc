
#include <iostream>
#include <umbrella/x86/ToString.hpp>

namespace umbrella::x86
{

std::ostream& operator<<(std::ostream& os, RegisterKind k)
{ return os << toString(k); }

std::ostream& operator<<(std::ostream& os, Register reg)
{ return os << toString(reg); }

std::ostream& operator<<(std::ostream& os, Opcode opcode)
{ return os << toString(opcode); }

}  // namespace umbrella::x86