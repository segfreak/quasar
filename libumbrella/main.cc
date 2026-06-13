
#include <iostream>

#include "umbrella/Display.hpp"
#include "umbrella/Instruction.hpp"
#include "umbrella/InstructionFactory.hpp"
#include "umbrella/Register.hpp"
#include "umbrella/ToString.hpp"
#include "umbrella/Type.hpp"
#include "umbrella/x86/Display.hpp"
#include "umbrella/x86/InstructionInfo.hpp"
#include "umbrella/x86/InstructionSet.hpp"
#include "umbrella/x86/Register.hpp"
#include "umbrella/x86/ToString.hpp"

int main()
{
    umbrella::Type        int32{umbrella::TypeKind::Int32};
    umbrella::Instruction t0 = umbrella::InstructionFactory::createAdd(
        umbrella::Register{2}, umbrella::Register{0},
        umbrella::Register{1});
    t0.verify();

    umbrella::Instruction t1 = umbrella::InstructionFactory::createSub(
        umbrella::Register{3}, umbrella::Register{0},
        umbrella::Register{1});
    t1.verify();

    auto op  = umbrella::x86::Opcode::Imul8r;
    auto src = umbrella::x86::Register{umbrella::x86::RegisterKind::Al};

    std::cout << op << " %" << src << "\n";

    auto info =
        umbrella::x86::InstructionInfo::get(umbrella::x86::Opcode::Imul8r);

    std::cout << "explicit operand kinds:\n";
    for (const auto& k : info.getExplicitOperandKinds())
    {
        std::cout << umbrella::x86::toString(k) << "\n";
    }

    std::cout << "implicit operands:\n";
    for (const auto& o : info.getImplicitOperands())
    {
        std::cout << umbrella::x86::toString(o) << "("
                  << umbrella::x86::toString(o.getKind().value()) << ")"
                  << " role: " << umbrella::toString(o.getRole()) << "\n";
    }

    return 0;
}
