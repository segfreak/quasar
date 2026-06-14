#include "umbrella/ToString.hpp"

#include <ranges>
#include <sstream>
#include <string_view>
#include <umbrella/Logging.hpp>
#include <umbrella/x86/Display.hpp>
#include <umbrella/x86/Instruction.hpp>
#include <umbrella/x86/InstructionSet.hpp>
#include <umbrella/x86/Operand.hpp>
#include <umbrella/x86/Register.hpp>
#include <umbrella/x86/ToString.hpp>
#include <utility>

namespace umbrella::x86
{

std::string toString(const Instruction& instr)
{
    std::ostringstream ss;

    ss << toString(instr.getOpcode());

    const auto& operands = instr.getOperands();
    if (!operands.empty())
    {
        ss << "  ";
        ss << toString(operands.front());

        for (const auto& operand : operands | std::views::drop(1))
        {
            ss << ", " << toString(operand);
        }
    }
    return ss.str();
}

std::string toString(const Operand& op)
{
    std::ostringstream ss;

    switch (*op.getKind())
    {
        case OperandKind::Register:
        {
            auto reg = op.getRegister().value();
            ss << toString(reg);
            break;
        }
        case OperandKind::Immediate:
        {
            auto imm = op.getImmediate().value();
            ss << "$0x" << std::hex << imm;
            break;
        }
        case OperandKind::Memory:
        {
            auto mem = op.getMemory().value();
            ss << toString(mem);
            break;
        }
    }

    return ss.str();
}

std::string_view toString(OperandKind k)
{
    switch (k)
    {
        case OperandKind::Register:
            return "r";
        case OperandKind::Immediate:
            return "imm";
        case OperandKind::Memory:
            return "m";
        default:
            errs("toString")
                << "unknown operand kind: " << static_cast<std::uint8_t>(k)
                << "\n";
            return "unknown";
    }
}

std::string toString(const Memory& mem)
{
    std::ostringstream oss;

    if (mem.getDisplacement() != 0 || mem.isAbsolute())
    {
        oss << mem.getDisplacement();
    }

    if (!mem.hasBase() && !mem.hasIndex()) { return oss.str(); }

    oss << "(";

    if (mem.hasBase()) { oss << toString(mem.getBase().value()); }

    if (mem.hasIndex())
    {
        oss << "," << toString(mem.getIndex().value()) << ","
            << static_cast<int>(mem.getScale());
    }

    oss << ")";

    return oss.str();
}
std::string_view toString(RegisterKind k)
{
    switch (k)
    {
        // 8-bit
        case RegisterKind::Al:
            return "al";
        case RegisterKind::Bl:
            return "bl";
        case RegisterKind::Cl:
            return "cl";
        case RegisterKind::Dl:
            return "dl";
        case RegisterKind::Ah:
            return "ah";
        case RegisterKind::Bh:
            return "bh";
        case RegisterKind::Ch:
            return "ch";
        case RegisterKind::Dh:
            return "dh";

        // 16-bit
        case RegisterKind::Ax:
            return "ax";
        case RegisterKind::Bx:
            return "bx";
        case RegisterKind::Cx:
            return "cx";
        case RegisterKind::Dx:
            return "dx";
        case RegisterKind::Sp:
            return "sp";
        case RegisterKind::Bp:
            return "bp";
        case RegisterKind::Si:
            return "si";
        case RegisterKind::Di:
            return "di";
        case RegisterKind::R8w:
            return "r8w";
        case RegisterKind::R9w:
            return "r9w";
        case RegisterKind::R10w:
            return "r10w";
        case RegisterKind::R11w:
            return "r11w";
        case RegisterKind::R12w:
            return "r12w";
        case RegisterKind::R13w:
            return "r13w";
        case RegisterKind::R14w:
            return "r14w";
        case RegisterKind::R15w:
            return "r15w";

        // 32-bit
        case RegisterKind::Eax:
            return "eax";
        case RegisterKind::Ebx:
            return "ebx";
        case RegisterKind::Ecx:
            return "ecx";
        case RegisterKind::Edx:
            return "edx";
        case RegisterKind::Esp:
            return "esp";
        case RegisterKind::Ebp:
            return "ebp";
        case RegisterKind::Esi:
            return "esi";
        case RegisterKind::Edi:
            return "edi";
        case RegisterKind::R8d:
            return "r8d";
        case RegisterKind::R9d:
            return "r9d";
        case RegisterKind::R10d:
            return "r10d";
        case RegisterKind::R11d:
            return "r11d";
        case RegisterKind::R12d:
            return "r12d";
        case RegisterKind::R13d:
            return "r13d";
        case RegisterKind::R14d:
            return "r14d";
        case RegisterKind::R15d:
            return "r15d";

        // 64-bit
        case RegisterKind::Rax:
            return "rax";
        case RegisterKind::Rbx:
            return "rbx";
        case RegisterKind::Rcx:
            return "rcx";
        case RegisterKind::Rdx:
            return "rdx";
        case RegisterKind::Rsp:
            return "rsp";
        case RegisterKind::Rbp:
            return "rbp";
        case RegisterKind::Rsi:
            return "rsi";
        case RegisterKind::Rdi:
            return "rdi";
        case RegisterKind::R8:
            return "r8";
        case RegisterKind::R9:
            return "r9";
        case RegisterKind::R10:
            return "r10";
        case RegisterKind::R11:
            return "r11";
        case RegisterKind::R12:
            return "r12";
        case RegisterKind::R13:
            return "r13";
        case RegisterKind::R14:
            return "r14";
        case RegisterKind::R15:
            return "r15";
        default:
            errs("toString")
                << "unknown register: " << static_cast<std::uint8_t>(k)
                << "\n";
            return "unknown";
    }
}

std::string toString(Register r)
{
    if (r.isVirtual()) { return toString(r.getVirtual().value()); }
    if (r.isPhysical())
    {
        return std::string{toString(r.getPhysical().value())};
    }
    std::unreachable();
}

std::string_view toString(X86Opcode opcode)
{
    switch (opcode)
    {
        case X86Opcode::Mov8rr:
        case X86Opcode::Mov8ri:
        case X86Opcode::Mov8rm:
        case X86Opcode::Mov8mr:
        case X86Opcode::Mov8mi:
            return "movb";
        case X86Opcode::Mov16rr:
        case X86Opcode::Mov16ri:
        case X86Opcode::Mov16rm:
        case X86Opcode::Mov16mr:
        case X86Opcode::Mov16mi:
            return "movw";
        case X86Opcode::Mov32rr:
        case X86Opcode::Mov32ri:
        case X86Opcode::Mov32rm:
        case X86Opcode::Mov32mr:
        case X86Opcode::Mov32mi:
            return "movl";
        case X86Opcode::Mov64rr:
        case X86Opcode::Mov64ri:
        case X86Opcode::Mov64rm:
        case X86Opcode::Mov64mr:
        case X86Opcode::Mov64mi:
            return "movq";

        case X86Opcode::Add8rr:
        case X86Opcode::Add8ri:
        case X86Opcode::Add8rm:
        case X86Opcode::Add8mr:
        case X86Opcode::Add8mi:
            return "addb";
        case X86Opcode::Add16rr:
        case X86Opcode::Add16ri:
        case X86Opcode::Add16rm:
        case X86Opcode::Add16mr:
        case X86Opcode::Add16mi:
            return "addw";
        case X86Opcode::Add32rr:
        case X86Opcode::Add32ri:
        case X86Opcode::Add32rm:
        case X86Opcode::Add32mr:
        case X86Opcode::Add32mi:
            return "addl";
        case X86Opcode::Add64rr:
        case X86Opcode::Add64ri:
        case X86Opcode::Add64rm:
        case X86Opcode::Add64mr:
        case X86Opcode::Add64mi:
            return "addq";

        case X86Opcode::Sub8rr:
        case X86Opcode::Sub8ri:
        case X86Opcode::Sub8rm:
        case X86Opcode::Sub8mr:
        case X86Opcode::Sub8mi:
            return "subb";
        case X86Opcode::Sub16rr:
        case X86Opcode::Sub16ri:
        case X86Opcode::Sub16rm:
        case X86Opcode::Sub16mr:
        case X86Opcode::Sub16mi:
            return "subw";
        case X86Opcode::Sub32rr:
        case X86Opcode::Sub32ri:
        case X86Opcode::Sub32rm:
        case X86Opcode::Sub32mr:
        case X86Opcode::Sub32mi:
            return "subl";
        case X86Opcode::Sub64rr:
        case X86Opcode::Sub64ri:
        case X86Opcode::Sub64rm:
        case X86Opcode::Sub64mr:
        case X86Opcode::Sub64mi:
            return "subq";

        case X86Opcode::Imul8r:
        case X86Opcode::Imul8rr:
            return "imulb";
        case X86Opcode::Imul16r:
        case X86Opcode::Imul16rr:
            return "imulw";
        case X86Opcode::Imul32r:
        case X86Opcode::Imul32rr:
            return "imull";
        case X86Opcode::Imul64r:
        case X86Opcode::Imul64rr:
            return "imulq";

        case X86Opcode::Div8r:
            return "divb";
        case X86Opcode::Div16r:
            return "divw";
        case X86Opcode::Div32r:
            return "divl";
        case X86Opcode::Div64r:
            return "divq";
        case X86Opcode::Idiv8r:
            return "idivb";
        case X86Opcode::Idiv16r:
            return "idivw";
        case X86Opcode::Idiv32r:
            return "idivl";
        case X86Opcode::Idiv64r:
            return "idivq";

        case X86Opcode::And8rr:
        case X86Opcode::And8ri:
        case X86Opcode::And8rm:
        case X86Opcode::And8mr:
        case X86Opcode::And8mi:
            return "andb";
        case X86Opcode::And16rr:
        case X86Opcode::And16ri:
        case X86Opcode::And16rm:
        case X86Opcode::And16mr:
        case X86Opcode::And16mi:
            return "andw";
        case X86Opcode::And32rr:
        case X86Opcode::And32ri:
        case X86Opcode::And32rm:
        case X86Opcode::And32mr:
        case X86Opcode::And32mi:
            return "andl";
        case X86Opcode::And64rr:
        case X86Opcode::And64ri:
        case X86Opcode::And64rm:
        case X86Opcode::And64mr:
        case X86Opcode::And64mi:
            return "andq";

        case X86Opcode::Or8rr:
        case X86Opcode::Or8ri:
        case X86Opcode::Or8rm:
        case X86Opcode::Or8mr:
        case X86Opcode::Or8mi:
            return "orb";
        case X86Opcode::Or16rr:
        case X86Opcode::Or16ri:
        case X86Opcode::Or16rm:
        case X86Opcode::Or16mr:
        case X86Opcode::Or16mi:
            return "orw";
        case X86Opcode::Or32rr:
        case X86Opcode::Or32ri:
        case X86Opcode::Or32rm:
        case X86Opcode::Or32mr:
        case X86Opcode::Or32mi:
            return "orl";
        case X86Opcode::Or64rr:
        case X86Opcode::Or64ri:
        case X86Opcode::Or64rm:
        case X86Opcode::Or64mr:
        case X86Opcode::Or64mi:
            return "orq";

        case X86Opcode::Xor8rr:
        case X86Opcode::Xor8ri:
        case X86Opcode::Xor8rm:
        case X86Opcode::Xor8mr:
        case X86Opcode::Xor8mi:
            return "xorb";
        case X86Opcode::Xor16rr:
        case X86Opcode::Xor16ri:
        case X86Opcode::Xor16rm:
        case X86Opcode::Xor16mr:
        case X86Opcode::Xor16mi:
            return "xorw";
        case X86Opcode::Xor32rr:
        case X86Opcode::Xor32ri:
        case X86Opcode::Xor32rm:
        case X86Opcode::Xor32mr:
        case X86Opcode::Xor32mi:
            return "xorl";
        case X86Opcode::Xor64rr:
        case X86Opcode::Xor64ri:
        case X86Opcode::Xor64rm:
        case X86Opcode::Xor64mr:
        case X86Opcode::Xor64mi:
            return "xorq";

        case X86Opcode::Shl8ri:
        case X86Opcode::Shl8rc:
        case X86Opcode::Shl8rm:
            return "shlb";
        case X86Opcode::Shl16ri:
        case X86Opcode::Shl16rc:
        case X86Opcode::Shl16rm:
            return "shlw";
        case X86Opcode::Shl32ri:
        case X86Opcode::Shl32rc:
        case X86Opcode::Shl32rm:
            return "shll";
        case X86Opcode::Shl64ri:
        case X86Opcode::Shl64rc:
        case X86Opcode::Shl64rm:
            return "shlq";

        case X86Opcode::Shr8ri:
        case X86Opcode::Shr8rc:
        case X86Opcode::Shr8rm:
            return "shrb";
        case X86Opcode::Shr16ri:
        case X86Opcode::Shr16rc:
        case X86Opcode::Shr16rm:
            return "shrw";
        case X86Opcode::Shr32ri:
        case X86Opcode::Shr32rc:
        case X86Opcode::Shr32rm:
            return "shrl";
        case X86Opcode::Shr64ri:
        case X86Opcode::Shr64rc:
        case X86Opcode::Shr64rm:
            return "shrq";

        case X86Opcode::Sar8ri:
        case X86Opcode::Sar8rc:
        case X86Opcode::Sar8rm:
            return "sarb";
        case X86Opcode::Sar16ri:
        case X86Opcode::Sar16rc:
        case X86Opcode::Sar16rm:
            return "sarw";
        case X86Opcode::Sar32ri:
        case X86Opcode::Sar32rc:
        case X86Opcode::Sar32rm:
            return "sarl";
        case X86Opcode::Sar64ri:
        case X86Opcode::Sar64rc:
        case X86Opcode::Sar64rm:
            return "sarq";

        case X86Opcode::Cmp8rr:
        case X86Opcode::Cmp8ri:
        case X86Opcode::Cmp8rm:
        case X86Opcode::Cmp8mr:
            return "cmpb";
        case X86Opcode::Cmp16rr:
        case X86Opcode::Cmp16ri:
        case X86Opcode::Cmp16rm:
        case X86Opcode::Cmp16mr:
            return "cmpw";
        case X86Opcode::Cmp32rr:
        case X86Opcode::Cmp32ri:
        case X86Opcode::Cmp32rm:
        case X86Opcode::Cmp32mr:
            return "cmpl";
        case X86Opcode::Cmp64rr:
        case X86Opcode::Cmp64ri:
        case X86Opcode::Cmp64rm:
        case X86Opcode::Cmp64mr:
            return "cmpq";

        case X86Opcode::Test8rr:
            return "testb";
        case X86Opcode::Test16rr:
            return "testw";
        case X86Opcode::Test32rr:
            return "testl";
        case X86Opcode::Test64rr:
            return "testq";

        case X86Opcode::Jmp:
            return "jmp";
        case X86Opcode::Je:
            return "je";
        case X86Opcode::Jne:
            return "jne";
        case X86Opcode::Jl:
            return "jl";
        case X86Opcode::Jle:
            return "jle";
        case X86Opcode::Jg:
            return "jg";
        case X86Opcode::Jge:
            return "jge";
        case X86Opcode::Call:
            return "call";
        case X86Opcode::Ret:
            return "ret";

        case X86Opcode::Sete:
            return "sete";
        case X86Opcode::Setne:
            return "setne";
        case X86Opcode::Setl:
            return "setl";
        case X86Opcode::Setle:
            return "setle";
        case X86Opcode::Setg:
            return "setg";
        case X86Opcode::Setge:
            return "setge";

        case X86Opcode::Movzx8_16:
            return "movzbw";
        case X86Opcode::Movzx8_32:
            return "movzbl";
        case X86Opcode::Movzx8_64:
            return "movzbq";
        case X86Opcode::Movzx16_32:
            return "movzwl";
        case X86Opcode::Movzx16_64:
            return "movzwq";
        case X86Opcode::Movzx32_64:
            return "movzlq";

        case X86Opcode::Movsx8_16:
            return "movsbw";
        case X86Opcode::Movsx8_32:
            return "movsbl";
        case X86Opcode::Movsx8_64:
            return "movsbq";
        case X86Opcode::Movsx16_32:
            return "movswl";
        case X86Opcode::Movsx16_64:
            return "movswq";
        case X86Opcode::Movsx32_64:
            return "movslq";

        case X86Opcode::Neg8r:
            return "negb";
        case X86Opcode::Neg16r:
            return "negw";
        case X86Opcode::Neg32r:
            return "negl";
        case X86Opcode::Neg64r:
            return "negq";

        case X86Opcode::Not8r:
            return "notb";
        case X86Opcode::Not16r:
            return "notw";
        case X86Opcode::Not32r:
            return "notl";
        case X86Opcode::Not64r:
            return "notq";

        case X86Opcode::Lea16rm:
            return "leaw";
        case X86Opcode::Lea32rm:
            return "leal";
        case X86Opcode::Lea64rm:
            return "leaq";
            // default:
            //     errs("toString")
            //         << "unknown opcode: " <<
            //         static_cast<std::uint8_t>(opcode)
            //         << "\n";
            //     return "unknown";
    }
}

}  // namespace umbrella::x86