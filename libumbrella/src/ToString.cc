#include <sstream>
#include <string>
#include <umbrella/Display.hpp>
#include <umbrella/Instruction.hpp>
#include <umbrella/Logging.hpp>
#include <umbrella/ToString.hpp>
#include <umbrella/Type.hpp>

namespace umbrella
{

std::string_view toString(Opcode opcode)
{
    switch (opcode)
    {
        case Opcode::Mov:
            return "mov";
        case Opcode::Add:
            return "add";
        case Opcode::Sub:
            return "sub";
        case Opcode::Ret:
            return "ret";
        default:
            errs("toString")
                << "unknown opcode: " << static_cast<std::uint8_t>(opcode)
                << "\n";
            return "unknown";
    }
}

std::string_view toString(OperandRole role)
{
    switch (role)
    {
        case OperandRole::Dst:
            return "dst";
        case OperandRole::Src:
            return "src";
        case OperandRole::DstSrc:
            return "dst/src";
        default:
            errs("toString") << "unknown operand role: "
                             << static_cast<std::uint8_t>(role) << "\n";
            return "unknown";
    }
}

std::string_view toString(TypeKind kind)
{
    switch (kind)
    {
        case TypeKind::Void:
            return "void";
        case TypeKind::Int8:
            return "int8";
        case TypeKind::Int16:
            return "int16";
        case TypeKind::Int32:
            return "int32";
        case TypeKind::Int64:
            return "int64";
        case TypeKind::Float32:
            return "float32";
        case TypeKind::Float64:
            return "float64";
        case TypeKind::Pointer:
            return "ptr";
        default:
            errs("toString")
                << "unknown type kind: " << static_cast<std::uint8_t>(kind)
                << "\n";
            return "unknown";
    }
}

std::string_view toString(Type type)
{ return toString(type.getKind()); }

std::string toString(const Instruction& instr)
{
    std::ostringstream oss;

    oss << toString(instr.getOpcode());

    for (const auto& operand : instr.getOperands())
    {
        oss << " %r" << operand.getRegister().getId();
    }

    return oss.str();
}

}  // namespace umbrella