#pragma once

#include <ostream>

namespace umbrella {

enum class Opcode : std::uint8_t;
enum class OperandRole : std::uint8_t;
enum class TypeKind : std::uint8_t;
struct Type;
struct Instruction;

std::ostream& operator<<(std::ostream& os, Opcode opcode);
std::ostream& operator<<(std::ostream& os, OperandRole role);
std::ostream& operator<<(std::ostream& os, TypeKind kind);
std::ostream& operator<<(std::ostream& os, Type type);
std::ostream& operator<<(std::ostream& os, const Instruction& instr);

}  // namespace umbrella
