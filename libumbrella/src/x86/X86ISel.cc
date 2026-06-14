
#include "umbrella/x86/X86ISel.hpp"

#include "umbrella/Instruction.hpp"
#include "umbrella/Logging.hpp"
#include "umbrella/ToString.hpp"
#include "umbrella/x86/InstructionSet.hpp"
#include "umbrella/x86/Operand.hpp"

namespace umbrella::x86
{

bool selectAdd(std::vector<Instruction>& out, const umbrella::Operand& dst,
               const umbrella::Operand& src1,
               const umbrella::Operand& src2)
{
}

bool selectSub(std::vector<Instruction>& out, const umbrella::Operand& dst,
               const umbrella::Operand& src1,
               const umbrella::Operand& src2)
{
}

std::vector<Instruction> X86ISel::select(
    const std::vector<umbrella::Instruction>& src)
{
    std::vector<Instruction> result;

    for (const auto& instr : src)
    {
        if (instr.verify())
        {
            errs("X86ISel") << "instruction verify failure, skip..\n";
            errs("X86ISel") << "your code is bogus at this point\n";
            continue;
        }

        switch (instr.getOpcode())
        {
            case Opcode::Add:
            {
                auto        operands = instr.getOperands();
                const auto& dst      = operands[0];
                const auto& src1     = operands[1];
                const auto& src2     = operands[2];
                if (!selectAdd(result, dst, src1, src2))
                {
                    errs("X86ISel") << "add selection failed\n";
                }
                break;
            }
            case Opcode::Sub:
            {
                auto        operands = instr.getOperands();
                const auto& dst      = operands[0];
                const auto& src1     = operands[1];
                const auto& src2     = operands[2];
                if (!selectSub(result, dst, src1, src2))
                {
                    errs("X86ISel") << "sub selection failed\n";
                }
                break;
            }

            default:
            {
                errs("X86ISel")
                    << "unhandled opcode: " << toString(instr.getOpcode())
                    << "\n";
                break;
            }
        }
    }

    return result;
}

}  // namespace umbrella::x86