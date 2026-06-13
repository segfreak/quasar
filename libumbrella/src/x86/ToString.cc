
#include <sstream>
#include <string_view>
#include <umbrella/Logging.hpp>
#include <umbrella/x86/InstructionSet.hpp>
#include <umbrella/x86/Operand.hpp>
#include <umbrella/x86/Register.hpp>
#include <umbrella/x86/ToString.hpp>

namespace umbrella::x86
{

std::string toString(const Operand& op)
{
    std::ostringstream ss;

    switch (*op.getKind())
    {
        case OperandKind::Register:
        {
            auto reg = op.getRegister().value();
            ss << "%" << toString(reg.getKind());
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

    if (mem.hasBase()) { oss << toString(mem.getBase()->getKind()); }

    if (mem.hasIndex())
    {
        oss << "," << toString(mem.getIndex()->getKind()) << ","
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

std::string_view toString(Register r)
{ return toString(r.getKind()); }

std::string_view toString(Opcode opcode)
{
    switch (opcode)
    {
        case Opcode::Mov8rr:
        case Opcode::Mov8ri:
        case Opcode::Mov8rm:
        case Opcode::Mov8mr:
        case Opcode::Mov8mi:
            return "movb";
        case Opcode::Mov16rr:
        case Opcode::Mov16ri:
        case Opcode::Mov16rm:
        case Opcode::Mov16mr:
        case Opcode::Mov16mi:
            return "movw";
        case Opcode::Mov32rr:
        case Opcode::Mov32ri:
        case Opcode::Mov32rm:
        case Opcode::Mov32mr:
        case Opcode::Mov32mi:
            return "movl";
        case Opcode::Mov64rr:
        case Opcode::Mov64ri:
        case Opcode::Mov64rm:
        case Opcode::Mov64mr:
        case Opcode::Mov64mi:
            return "movq";

        case Opcode::Add8rr:
        case Opcode::Add8ri:
        case Opcode::Add8rm:
        case Opcode::Add8mr:
        case Opcode::Add8mi:
            return "addb";
        case Opcode::Add16rr:
        case Opcode::Add16ri:
        case Opcode::Add16rm:
        case Opcode::Add16mr:
        case Opcode::Add16mi:
            return "addw";
        case Opcode::Add32rr:
        case Opcode::Add32ri:
        case Opcode::Add32rm:
        case Opcode::Add32mr:
        case Opcode::Add32mi:
            return "addl";
        case Opcode::Add64rr:
        case Opcode::Add64ri:
        case Opcode::Add64rm:
        case Opcode::Add64mr:
        case Opcode::Add64mi:
            return "addq";

        case Opcode::Sub8rr:
        case Opcode::Sub8ri:
        case Opcode::Sub8rm:
        case Opcode::Sub8mr:
        case Opcode::Sub8mi:
            return "subb";
        case Opcode::Sub16rr:
        case Opcode::Sub16ri:
        case Opcode::Sub16rm:
        case Opcode::Sub16mr:
        case Opcode::Sub16mi:
            return "subw";
        case Opcode::Sub32rr:
        case Opcode::Sub32ri:
        case Opcode::Sub32rm:
        case Opcode::Sub32mr:
        case Opcode::Sub32mi:
            return "subl";
        case Opcode::Sub64rr:
        case Opcode::Sub64ri:
        case Opcode::Sub64rm:
        case Opcode::Sub64mr:
        case Opcode::Sub64mi:
            return "subq";

        case Opcode::Imul8r:
        case Opcode::Imul8rr:
            return "imulb";
        case Opcode::Imul16r:
        case Opcode::Imul16rr:
            return "imulw";
        case Opcode::Imul32r:
        case Opcode::Imul32rr:
            return "imull";
        case Opcode::Imul64r:
        case Opcode::Imul64rr:
            return "imulq";

        case Opcode::Div8r:
            return "divb";
        case Opcode::Div16r:
            return "divw";
        case Opcode::Div32r:
            return "divl";
        case Opcode::Div64r:
            return "divq";
        case Opcode::Idiv8r:
            return "idivb";
        case Opcode::Idiv16r:
            return "idivw";
        case Opcode::Idiv32r:
            return "idivl";
        case Opcode::Idiv64r:
            return "idivq";

        case Opcode::And8rr:
        case Opcode::And8ri:
        case Opcode::And8rm:
        case Opcode::And8mr:
        case Opcode::And8mi:
            return "andb";
        case Opcode::And16rr:
        case Opcode::And16ri:
        case Opcode::And16rm:
        case Opcode::And16mr:
        case Opcode::And16mi:
            return "andw";
        case Opcode::And32rr:
        case Opcode::And32ri:
        case Opcode::And32rm:
        case Opcode::And32mr:
        case Opcode::And32mi:
            return "andl";
        case Opcode::And64rr:
        case Opcode::And64ri:
        case Opcode::And64rm:
        case Opcode::And64mr:
        case Opcode::And64mi:
            return "andq";

        case Opcode::Or8rr:
        case Opcode::Or8ri:
        case Opcode::Or8rm:
        case Opcode::Or8mr:
        case Opcode::Or8mi:
            return "orb";
        case Opcode::Or16rr:
        case Opcode::Or16ri:
        case Opcode::Or16rm:
        case Opcode::Or16mr:
        case Opcode::Or16mi:
            return "orw";
        case Opcode::Or32rr:
        case Opcode::Or32ri:
        case Opcode::Or32rm:
        case Opcode::Or32mr:
        case Opcode::Or32mi:
            return "orl";
        case Opcode::Or64rr:
        case Opcode::Or64ri:
        case Opcode::Or64rm:
        case Opcode::Or64mr:
        case Opcode::Or64mi:
            return "orq";

        case Opcode::Xor8rr:
        case Opcode::Xor8ri:
        case Opcode::Xor8rm:
        case Opcode::Xor8mr:
        case Opcode::Xor8mi:
            return "xorb";
        case Opcode::Xor16rr:
        case Opcode::Xor16ri:
        case Opcode::Xor16rm:
        case Opcode::Xor16mr:
        case Opcode::Xor16mi:
            return "xorw";
        case Opcode::Xor32rr:
        case Opcode::Xor32ri:
        case Opcode::Xor32rm:
        case Opcode::Xor32mr:
        case Opcode::Xor32mi:
            return "xorl";
        case Opcode::Xor64rr:
        case Opcode::Xor64ri:
        case Opcode::Xor64rm:
        case Opcode::Xor64mr:
        case Opcode::Xor64mi:
            return "xorq";

        case Opcode::Shl8ri:
        case Opcode::Shl8rc:
        case Opcode::Shl8rm:
            return "shlb";
        case Opcode::Shl16ri:
        case Opcode::Shl16rc:
        case Opcode::Shl16rm:
            return "shlw";
        case Opcode::Shl32ri:
        case Opcode::Shl32rc:
        case Opcode::Shl32rm:
            return "shll";
        case Opcode::Shl64ri:
        case Opcode::Shl64rc:
        case Opcode::Shl64rm:
            return "shlq";

        case Opcode::Shr8ri:
        case Opcode::Shr8rc:
        case Opcode::Shr8rm:
            return "shrb";
        case Opcode::Shr16ri:
        case Opcode::Shr16rc:
        case Opcode::Shr16rm:
            return "shrw";
        case Opcode::Shr32ri:
        case Opcode::Shr32rc:
        case Opcode::Shr32rm:
            return "shrl";
        case Opcode::Shr64ri:
        case Opcode::Shr64rc:
        case Opcode::Shr64rm:
            return "shrq";

        case Opcode::Sar8ri:
        case Opcode::Sar8rc:
        case Opcode::Sar8rm:
            return "sarb";
        case Opcode::Sar16ri:
        case Opcode::Sar16rc:
        case Opcode::Sar16rm:
            return "sarw";
        case Opcode::Sar32ri:
        case Opcode::Sar32rc:
        case Opcode::Sar32rm:
            return "sarl";
        case Opcode::Sar64ri:
        case Opcode::Sar64rc:
        case Opcode::Sar64rm:
            return "sarq";

        case Opcode::Cmp8rr:
        case Opcode::Cmp8ri:
        case Opcode::Cmp8rm:
        case Opcode::Cmp8mr:
            return "cmpb";
        case Opcode::Cmp16rr:
        case Opcode::Cmp16ri:
        case Opcode::Cmp16rm:
        case Opcode::Cmp16mr:
            return "cmpw";
        case Opcode::Cmp32rr:
        case Opcode::Cmp32ri:
        case Opcode::Cmp32rm:
        case Opcode::Cmp32mr:
            return "cmpl";
        case Opcode::Cmp64rr:
        case Opcode::Cmp64ri:
        case Opcode::Cmp64rm:
        case Opcode::Cmp64mr:
            return "cmpq";

        case Opcode::Test8rr:
            return "testb";
        case Opcode::Test16rr:
            return "testw";
        case Opcode::Test32rr:
            return "testl";
        case Opcode::Test64rr:
            return "testq";

        case Opcode::Jmp:
            return "jmp";
        case Opcode::Je:
            return "je";
        case Opcode::Jne:
            return "jne";
        case Opcode::Jl:
            return "jl";
        case Opcode::Jle:
            return "jle";
        case Opcode::Jg:
            return "jg";
        case Opcode::Jge:
            return "jge";
        case Opcode::Call:
            return "call";
        case Opcode::Ret:
            return "ret";

        case Opcode::Sete:
            return "sete";
        case Opcode::Setne:
            return "setne";
        case Opcode::Setl:
            return "setl";
        case Opcode::Setle:
            return "setle";
        case Opcode::Setg:
            return "setg";
        case Opcode::Setge:
            return "setge";

        case Opcode::Movzx8_16:
            return "movzbw";
        case Opcode::Movzx8_32:
            return "movzbl";
        case Opcode::Movzx8_64:
            return "movzbq";
        case Opcode::Movzx16_32:
            return "movzwl";
        case Opcode::Movzx16_64:
            return "movzwq";
        case Opcode::Movzx32_64:
            return "movzlq";

        case Opcode::Movsx8_16:
            return "movsbw";
        case Opcode::Movsx8_32:
            return "movsbl";
        case Opcode::Movsx8_64:
            return "movsbq";
        case Opcode::Movsx16_32:
            return "movswl";
        case Opcode::Movsx16_64:
            return "movswq";
        case Opcode::Movsx32_64:
            return "movslq";

        case Opcode::Neg8r:
            return "negb";
        case Opcode::Neg16r:
            return "negw";
        case Opcode::Neg32r:
            return "negl";
        case Opcode::Neg64r:
            return "negq";

        case Opcode::Not8r:
            return "notb";
        case Opcode::Not16r:
            return "notw";
        case Opcode::Not32r:
            return "notl";
        case Opcode::Not64r:
            return "notq";
            // default:
            //     errs("toString")
            //         << "unknown opcode: " <<
            //         static_cast<std::uint8_t>(opcode)
            //         << "\n";
            //     return "unknown";
    }
}

}  // namespace umbrella::x86