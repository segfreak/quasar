
#include <umbrella/ToString.hpp>

namespace umbrella {

std::ostream& operator<<(std::ostream& os, Opcode opcode)
{ return os << toString(opcode); }

std::ostream& operator<<(std::ostream& os, OperandRole role)
{ return os << toString(role); }

std::ostream& operator<<(std::ostream& os, TypeKind kind)
{ return os << toString(kind); }

std::ostream& operator<<(std::ostream& os, Type type)
{ return os << toString(type); }

std::ostream& operator<<(std::ostream& os, const Instruction& instr)
{ return os << toString(instr); }

}  // namespace umbrella