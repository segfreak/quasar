
#include <iostream>

#include "umbrella/Display.hpp"
#include "umbrella/Instruction.hpp"
#include "umbrella/InstructionFactory.hpp"
#include "umbrella/ToString.hpp"
#include "umbrella/Type.hpp"
#include "umbrella/VirtualRegister.hpp"
#include "umbrella/x86/Display.hpp"
#include "umbrella/x86/Instruction.hpp"
#include "umbrella/x86/InstructionInfo.hpp"
#include "umbrella/x86/Operand.hpp"
#include "umbrella/x86/ToString.hpp"
#include "umbrella/x86/X86ISel.hpp"

int main()
{
    umbrella::Type        int32{umbrella::TypeKind::Int32};
    umbrella::Instruction t0 = umbrella::InstructionFactory::createAdd(
        umbrella::VirtualRegister{2, umbrella::TypeKind::Int8},
        umbrella::VirtualRegister{0, umbrella::TypeKind::Int8},
        umbrella::VirtualRegister{1, umbrella::TypeKind::Int8});
    t0.verify();

    umbrella::Instruction t1 = umbrella::InstructionFactory::createAdd(
        umbrella::VirtualRegister{2, umbrella::TypeKind::Int32},
        umbrella::VirtualRegister{0, umbrella::TypeKind::Int32},
        umbrella::VirtualRegister{1, umbrella::TypeKind::Int32});
    t0.verify();

    auto                   src = {t1};
    umbrella::x86::X86ISel isel;
    std::cout << "before isel:\n";
    for (const auto& instr : src) { std::cout << "  " << instr << "\n"; }
    auto post_isel = isel.select(src);
    std::cout << "after isel:\n";
    for (const auto& instr : post_isel)
    {
        std::cout << "  " << instr << "\n";
    }

    for (const auto& instr : post_isel)
    {
        std::cout << "analysis of " << instr << "\n";

        const auto& info = instr.getInfo();

        std::cout << "explicit operand kinds:\n";
        for (const auto& k : info.getExplicitOperandKinds())
        {
            std::cout << k << "\n";
        }

        std::cout << "implicit operands:\n";
        for (const auto& o : info.getImplicitOperands())
        {
            std::cout << umbrella::x86::toString(o) << "("
                      << umbrella::x86::toString(o.getKind().value())
                      << ")"
                      << " role: " << umbrella::toString(o.getRole())
                      << "\n";
        }
    }

    return 0;
}
