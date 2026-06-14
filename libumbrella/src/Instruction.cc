
#include <cstddef>
#include <umbrella/Display.hpp>
#include <umbrella/Instruction.hpp>
#include <umbrella/support/Logging.hpp>

namespace umbrella {

std::vector<OperandRole> getExpectedOperandRolesFor(Opcode opcode)
{
    switch (opcode) {
        case Opcode::Mov:
            return {OperandRole::Dst, OperandRole::Src};
        case Opcode::Add:
        case Opcode::Sub:
            return {OperandRole::Dst, OperandRole::Src, OperandRole::Src};
        case Opcode::Ret:
            return {OperandRole::Src};
        default:
            errs("getExpectedOperandRolesFor")
                << "unknown opcode: " << static_cast<std::uint8_t>(opcode)
                << "\n";
            return {};
    }
}

bool Instruction::verify() const
{
    auto expectedRoles = getExpectedOperandRolesFor(getOpcode());
    auto operands      = getOperands();

    if (operands.size() != expectedRoles.size()) {
        errs("verify") << "operands count mismatch\n";
        errs("verify") << " operands.size() != expectedRoles.size()\n";
        return false;
    }

    for (std::size_t i = 0; i < operands.size(); ++i) {
        const auto& operandRole  = operands.at(i).getRole();
        const auto& expectedRole = expectedRoles.at(i);

        if (operandRole != expectedRole) {
            errs("verify") << "operand role mismatch\n";
            errs("verify") << "got: " << operandRole << ", ";
            errs("verify") << "expected: " << expectedRole << "\n";
            return false;
        }
    }

    return false;
}

}  // namespace umbrella