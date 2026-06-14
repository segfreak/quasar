#include <umbrella/x86/InstructionInfo.hpp>
#include <umbrella/x86/InstructionSet.hpp>
#include <umbrella/x86/Operand.hpp>

namespace umbrella::x86
{

std::span<const OperandKind> InstructionInfo::getExplicitOperandKinds()
    const
{
    static const OperandKind              r[]   = {OperandKind::Register};
    static const OperandKind              rr[]  = {OperandKind::Register,
                                                   OperandKind::Register};
    static const OperandKind              ri[]  = {OperandKind::Register,
                                                   OperandKind::Immediate};
    static const OperandKind              rm[]  = {OperandKind::Register,
                                                   OperandKind::Memory};
    static const OperandKind              mr[]  = {OperandKind::Memory,
                                                   OperandKind::Register};
    static const OperandKind              mi[]  = {OperandKind::Memory,
                                                   OperandKind::Immediate};
    static const std::vector<OperandKind> empty = {};

    switch (opcode_)
    {
        case X86Opcode::Imul8r:
        case X86Opcode::Imul16r:
        case X86Opcode::Imul32r:
        case X86Opcode::Imul64r:
        case X86Opcode::Div8r:
        case X86Opcode::Div16r:
        case X86Opcode::Div32r:
        case X86Opcode::Div64r:
        case X86Opcode::Idiv8r:
        case X86Opcode::Idiv16r:
        case X86Opcode::Idiv32r:
        case X86Opcode::Idiv64r:
        case X86Opcode::Neg8r:
        case X86Opcode::Neg16r:
        case X86Opcode::Neg32r:
        case X86Opcode::Neg64r:
        case X86Opcode::Not8r:
        case X86Opcode::Not16r:
        case X86Opcode::Not32r:
        case X86Opcode::Not64r:
        case X86Opcode::Sete:
        case X86Opcode::Setne:
        case X86Opcode::Setl:
        case X86Opcode::Setle:
        case X86Opcode::Setg:
        case X86Opcode::Setge:
            return r;

        case X86Opcode::Mov8rr:
        case X86Opcode::Mov16rr:
        case X86Opcode::Mov32rr:
        case X86Opcode::Mov64rr:
        case X86Opcode::Add8rr:
        case X86Opcode::Add16rr:
        case X86Opcode::Add32rr:
        case X86Opcode::Add64rr:
        case X86Opcode::Sub8rr:
        case X86Opcode::Sub16rr:
        case X86Opcode::Sub32rr:
        case X86Opcode::Sub64rr:
        case X86Opcode::And8rr:
        case X86Opcode::And16rr:
        case X86Opcode::And32rr:
        case X86Opcode::And64rr:
        case X86Opcode::Or8rr:
        case X86Opcode::Or16rr:
        case X86Opcode::Or32rr:
        case X86Opcode::Or64rr:
        case X86Opcode::Xor8rr:
        case X86Opcode::Xor16rr:
        case X86Opcode::Xor32rr:
        case X86Opcode::Xor64rr:
        case X86Opcode::Cmp8rr:
        case X86Opcode::Cmp16rr:
        case X86Opcode::Cmp32rr:
        case X86Opcode::Cmp64rr:
        case X86Opcode::Test8rr:
        case X86Opcode::Test16rr:
        case X86Opcode::Test32rr:
        case X86Opcode::Test64rr:
        case X86Opcode::Imul8rr:
        case X86Opcode::Imul16rr:
        case X86Opcode::Imul32rr:
        case X86Opcode::Imul64rr:
        case X86Opcode::Movzx8_16:
        case X86Opcode::Movzx8_32:
        case X86Opcode::Movzx8_64:
        case X86Opcode::Movzx16_32:
        case X86Opcode::Movzx16_64:
        case X86Opcode::Movzx32_64:
        case X86Opcode::Movsx8_16:
        case X86Opcode::Movsx8_32:
        case X86Opcode::Movsx8_64:
        case X86Opcode::Movsx16_32:
        case X86Opcode::Movsx16_64:
        case X86Opcode::Movsx32_64:
            return rr;

        case X86Opcode::Mov8ri:
        case X86Opcode::Mov16ri:
        case X86Opcode::Mov32ri:
        case X86Opcode::Mov64ri:
        case X86Opcode::Add8ri:
        case X86Opcode::Add16ri:
        case X86Opcode::Add32ri:
        case X86Opcode::Add64ri:
        case X86Opcode::Sub8ri:
        case X86Opcode::Sub16ri:
        case X86Opcode::Sub32ri:
        case X86Opcode::Sub64ri:
        case X86Opcode::And8ri:
        case X86Opcode::And16ri:
        case X86Opcode::And32ri:
        case X86Opcode::And64ri:
        case X86Opcode::Or8ri:
        case X86Opcode::Or16ri:
        case X86Opcode::Or32ri:
        case X86Opcode::Or64ri:
        case X86Opcode::Xor8ri:
        case X86Opcode::Xor16ri:
        case X86Opcode::Xor32ri:
        case X86Opcode::Xor64ri:
        case X86Opcode::Cmp8ri:
        case X86Opcode::Cmp16ri:
        case X86Opcode::Cmp32ri:
        case X86Opcode::Cmp64ri:
        case X86Opcode::Shl8ri:
        case X86Opcode::Shl16ri:
        case X86Opcode::Shl32ri:
        case X86Opcode::Shl64ri:
        case X86Opcode::Shr8ri:
        case X86Opcode::Shr16ri:
        case X86Opcode::Shr32ri:
        case X86Opcode::Shr64ri:
        case X86Opcode::Sar8ri:
        case X86Opcode::Sar16ri:
        case X86Opcode::Sar32ri:
        case X86Opcode::Sar64ri:
            return ri;

        case X86Opcode::Mov8rm:
        case X86Opcode::Mov16rm:
        case X86Opcode::Mov32rm:
        case X86Opcode::Mov64rm:
        case X86Opcode::Add8rm:
        case X86Opcode::Add16rm:
        case X86Opcode::Add32rm:
        case X86Opcode::Add64rm:
        case X86Opcode::Sub8rm:
        case X86Opcode::Sub16rm:
        case X86Opcode::Sub32rm:
        case X86Opcode::Sub64rm:
        case X86Opcode::And8rm:
        case X86Opcode::And16rm:
        case X86Opcode::And32rm:
        case X86Opcode::And64rm:
        case X86Opcode::Or8rm:
        case X86Opcode::Or16rm:
        case X86Opcode::Or32rm:
        case X86Opcode::Or64rm:
        case X86Opcode::Xor8rm:
        case X86Opcode::Xor16rm:
        case X86Opcode::Xor32rm:
        case X86Opcode::Xor64rm:
        case X86Opcode::Cmp8rm:
        case X86Opcode::Cmp16rm:
        case X86Opcode::Cmp32rm:
        case X86Opcode::Cmp64rm:
        case X86Opcode::Shl8rm:
        case X86Opcode::Shl16rm:
        case X86Opcode::Shl32rm:
        case X86Opcode::Shl64rm:
        case X86Opcode::Shr8rm:
        case X86Opcode::Shr16rm:
        case X86Opcode::Shr32rm:
        case X86Opcode::Shr64rm:
        case X86Opcode::Sar8rm:
        case X86Opcode::Sar16rm:
        case X86Opcode::Sar32rm:
        case X86Opcode::Sar64rm:
        case X86Opcode::Lea16rm:
        case X86Opcode::Lea32rm:
        case X86Opcode::Lea64rm:
            return rm;

        case X86Opcode::Mov8mr:
        case X86Opcode::Mov16mr:
        case X86Opcode::Mov32mr:
        case X86Opcode::Mov64mr:
        case X86Opcode::Add8mr:
        case X86Opcode::Add16mr:
        case X86Opcode::Add32mr:
        case X86Opcode::Add64mr:
        case X86Opcode::Sub8mr:
        case X86Opcode::Sub16mr:
        case X86Opcode::Sub32mr:
        case X86Opcode::Sub64mr:
        case X86Opcode::And8mr:
        case X86Opcode::And16mr:
        case X86Opcode::And32mr:
        case X86Opcode::And64mr:
        case X86Opcode::Or8mr:
        case X86Opcode::Or16mr:
        case X86Opcode::Or32mr:
        case X86Opcode::Or64mr:
        case X86Opcode::Xor8mr:
        case X86Opcode::Xor16mr:
        case X86Opcode::Xor32mr:
        case X86Opcode::Xor64mr:
        case X86Opcode::Cmp8mr:
        case X86Opcode::Cmp16mr:
        case X86Opcode::Cmp32mr:
        case X86Opcode::Cmp64mr:
            return mr;

        case X86Opcode::Mov8mi:
        case X86Opcode::Mov16mi:
        case X86Opcode::Mov32mi:
        case X86Opcode::Mov64mi:
        case X86Opcode::Add8mi:
        case X86Opcode::Add16mi:
        case X86Opcode::Add32mi:
        case X86Opcode::Add64mi:
        case X86Opcode::Sub8mi:
        case X86Opcode::Sub16mi:
        case X86Opcode::Sub32mi:
        case X86Opcode::Sub64mi:
        case X86Opcode::And8mi:
        case X86Opcode::And16mi:
        case X86Opcode::And32mi:
        case X86Opcode::And64mi:
        case X86Opcode::Or8mi:
        case X86Opcode::Or16mi:
        case X86Opcode::Or32mi:
        case X86Opcode::Or64mi:
        case X86Opcode::Xor8mi:
        case X86Opcode::Xor16mi:
        case X86Opcode::Xor32mi:
        case X86Opcode::Xor64mi:
            return mi;

        case X86Opcode::Shl8rc:
        case X86Opcode::Shl16rc:
        case X86Opcode::Shl32rc:
        case X86Opcode::Shl64rc:
        case X86Opcode::Shr8rc:
        case X86Opcode::Shr16rc:
        case X86Opcode::Shr32rc:
        case X86Opcode::Shr64rc:
        case X86Opcode::Sar8rc:
        case X86Opcode::Sar16rc:
        case X86Opcode::Sar32rc:
        case X86Opcode::Sar64rc:
            return r;

        case X86Opcode::Jmp:
        case X86Opcode::Je:
        case X86Opcode::Jne:
        case X86Opcode::Jl:
        case X86Opcode::Jle:
        case X86Opcode::Jg:
        case X86Opcode::Jge:
        case X86Opcode::Call:
        case X86Opcode::Ret:
            return empty;
    }
    return empty;
}

std::vector<Operand> InstructionInfo::getImplicitOperands() const
{
    std::vector<Operand> implicit;

    switch (opcode_)
    {
        case X86Opcode::Imul8r:
            implicit.emplace_back(Register(RegisterKind::Al),
                                  OperandRole::DstSrc);
            implicit.emplace_back(Register(RegisterKind::Ah),
                                  OperandRole::Dst);
            break;
        case X86Opcode::Imul16r:
            implicit.emplace_back(Register(RegisterKind::Ax),
                                  OperandRole::DstSrc);
            implicit.emplace_back(Register(RegisterKind::Dx),
                                  OperandRole::Dst);
            break;
        case X86Opcode::Imul32r:
            implicit.emplace_back(Register(RegisterKind::Eax),
                                  OperandRole::DstSrc);
            implicit.emplace_back(Register(RegisterKind::Edx),
                                  OperandRole::Dst);
            break;
        case X86Opcode::Imul64r:
            implicit.emplace_back(Register(RegisterKind::Rax),
                                  OperandRole::DstSrc);
            implicit.emplace_back(Register(RegisterKind::Rdx),
                                  OperandRole::Dst);
            break;

        case X86Opcode::Div8r:
        case X86Opcode::Idiv8r:
            implicit.emplace_back(Register(RegisterKind::Al),
                                  OperandRole::DstSrc);
            implicit.emplace_back(Register(RegisterKind::Ah),
                                  OperandRole::DstSrc);
            break;
        case X86Opcode::Div16r:
        case X86Opcode::Idiv16r:
            implicit.emplace_back(Register(RegisterKind::Ax),
                                  OperandRole::DstSrc);
            implicit.emplace_back(Register(RegisterKind::Dx),
                                  OperandRole::DstSrc);
            break;
        case X86Opcode::Div32r:
        case X86Opcode::Idiv32r:
            implicit.emplace_back(Register(RegisterKind::Eax),
                                  OperandRole::DstSrc);
            implicit.emplace_back(Register(RegisterKind::Edx),
                                  OperandRole::DstSrc);
            break;
        case X86Opcode::Div64r:
        case X86Opcode::Idiv64r:
            implicit.emplace_back(Register(RegisterKind::Rax),
                                  OperandRole::DstSrc);
            implicit.emplace_back(Register(RegisterKind::Rdx),
                                  OperandRole::DstSrc);
            break;

        case X86Opcode::Shl8rc:
        case X86Opcode::Shl16rc:
        case X86Opcode::Shl32rc:
        case X86Opcode::Shl64rc:
        case X86Opcode::Shr8rc:
        case X86Opcode::Shr16rc:
        case X86Opcode::Shr32rc:
        case X86Opcode::Shr64rc:
        case X86Opcode::Sar8rc:
        case X86Opcode::Sar16rc:
        case X86Opcode::Sar32rc:
        case X86Opcode::Sar64rc:
            implicit.emplace_back(Register(RegisterKind::Cl),
                                  OperandRole::Src);
            break;

        case X86Opcode::Call:
        case X86Opcode::Ret:
            implicit.emplace_back(Register(RegisterKind::Rsp),
                                  OperandRole::DstSrc);
            break;

        default:
            break;
    }

    return implicit;
}

}  // namespace umbrella::x86