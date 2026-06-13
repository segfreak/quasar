#include <umbrella/x86/InstructionInfo.hpp>
#include <umbrella/x86/InstructionSet.hpp>
#include <umbrella/x86/Operand.hpp>

namespace umbrella::x86
{

std::span<const OperandKind> InstructionInfo::getExplicitOperandKinds()
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
        case Opcode::Imul8r:
        case Opcode::Imul16r:
        case Opcode::Imul32r:
        case Opcode::Imul64r:
        case Opcode::Div8r:
        case Opcode::Div16r:
        case Opcode::Div32r:
        case Opcode::Div64r:
        case Opcode::Idiv8r:
        case Opcode::Idiv16r:
        case Opcode::Idiv32r:
        case Opcode::Idiv64r:
        case Opcode::Neg8r:
        case Opcode::Neg16r:
        case Opcode::Neg32r:
        case Opcode::Neg64r:
        case Opcode::Not8r:
        case Opcode::Not16r:
        case Opcode::Not32r:
        case Opcode::Not64r:
        case Opcode::Sete:
        case Opcode::Setne:
        case Opcode::Setl:
        case Opcode::Setle:
        case Opcode::Setg:
        case Opcode::Setge:
            return r;

        case Opcode::Mov8rr:
        case Opcode::Mov16rr:
        case Opcode::Mov32rr:
        case Opcode::Mov64rr:
        case Opcode::Add8rr:
        case Opcode::Add16rr:
        case Opcode::Add32rr:
        case Opcode::Add64rr:
        case Opcode::Sub8rr:
        case Opcode::Sub16rr:
        case Opcode::Sub32rr:
        case Opcode::Sub64rr:
        case Opcode::And8rr:
        case Opcode::And16rr:
        case Opcode::And32rr:
        case Opcode::And64rr:
        case Opcode::Or8rr:
        case Opcode::Or16rr:
        case Opcode::Or32rr:
        case Opcode::Or64rr:
        case Opcode::Xor8rr:
        case Opcode::Xor16rr:
        case Opcode::Xor32rr:
        case Opcode::Xor64rr:
        case Opcode::Cmp8rr:
        case Opcode::Cmp16rr:
        case Opcode::Cmp32rr:
        case Opcode::Cmp64rr:
        case Opcode::Test8rr:
        case Opcode::Test16rr:
        case Opcode::Test32rr:
        case Opcode::Test64rr:
        case Opcode::Imul8rr:
        case Opcode::Imul16rr:
        case Opcode::Imul32rr:
        case Opcode::Imul64rr:
        case Opcode::Movzx8_16:
        case Opcode::Movzx8_32:
        case Opcode::Movzx8_64:
        case Opcode::Movzx16_32:
        case Opcode::Movzx16_64:
        case Opcode::Movzx32_64:
        case Opcode::Movsx8_16:
        case Opcode::Movsx8_32:
        case Opcode::Movsx8_64:
        case Opcode::Movsx16_32:
        case Opcode::Movsx16_64:
        case Opcode::Movsx32_64:
            return rr;

        case Opcode::Mov8ri:
        case Opcode::Mov16ri:
        case Opcode::Mov32ri:
        case Opcode::Mov64ri:
        case Opcode::Add8ri:
        case Opcode::Add16ri:
        case Opcode::Add32ri:
        case Opcode::Add64ri:
        case Opcode::Sub8ri:
        case Opcode::Sub16ri:
        case Opcode::Sub32ri:
        case Opcode::Sub64ri:
        case Opcode::And8ri:
        case Opcode::And16ri:
        case Opcode::And32ri:
        case Opcode::And64ri:
        case Opcode::Or8ri:
        case Opcode::Or16ri:
        case Opcode::Or32ri:
        case Opcode::Or64ri:
        case Opcode::Xor8ri:
        case Opcode::Xor16ri:
        case Opcode::Xor32ri:
        case Opcode::Xor64ri:
        case Opcode::Cmp8ri:
        case Opcode::Cmp16ri:
        case Opcode::Cmp32ri:
        case Opcode::Cmp64ri:
        case Opcode::Shl8ri:
        case Opcode::Shl16ri:
        case Opcode::Shl32ri:
        case Opcode::Shl64ri:
        case Opcode::Shr8ri:
        case Opcode::Shr16ri:
        case Opcode::Shr32ri:
        case Opcode::Shr64ri:
        case Opcode::Sar8ri:
        case Opcode::Sar16ri:
        case Opcode::Sar32ri:
        case Opcode::Sar64ri:
            return ri;

        case Opcode::Mov8rm:
        case Opcode::Mov16rm:
        case Opcode::Mov32rm:
        case Opcode::Mov64rm:
        case Opcode::Add8rm:
        case Opcode::Add16rm:
        case Opcode::Add32rm:
        case Opcode::Add64rm:
        case Opcode::Sub8rm:
        case Opcode::Sub16rm:
        case Opcode::Sub32rm:
        case Opcode::Sub64rm:
        case Opcode::And8rm:
        case Opcode::And16rm:
        case Opcode::And32rm:
        case Opcode::And64rm:
        case Opcode::Or8rm:
        case Opcode::Or16rm:
        case Opcode::Or32rm:
        case Opcode::Or64rm:
        case Opcode::Xor8rm:
        case Opcode::Xor16rm:
        case Opcode::Xor32rm:
        case Opcode::Xor64rm:
        case Opcode::Cmp8rm:
        case Opcode::Cmp16rm:
        case Opcode::Cmp32rm:
        case Opcode::Cmp64rm:
        case Opcode::Shl8rm:
        case Opcode::Shl16rm:
        case Opcode::Shl32rm:
        case Opcode::Shl64rm:
        case Opcode::Shr8rm:
        case Opcode::Shr16rm:
        case Opcode::Shr32rm:
        case Opcode::Shr64rm:
        case Opcode::Sar8rm:
        case Opcode::Sar16rm:
        case Opcode::Sar32rm:
        case Opcode::Sar64rm:
            return rm;

        case Opcode::Mov8mr:
        case Opcode::Mov16mr:
        case Opcode::Mov32mr:
        case Opcode::Mov64mr:
        case Opcode::Add8mr:
        case Opcode::Add16mr:
        case Opcode::Add32mr:
        case Opcode::Add64mr:
        case Opcode::Sub8mr:
        case Opcode::Sub16mr:
        case Opcode::Sub32mr:
        case Opcode::Sub64mr:
        case Opcode::And8mr:
        case Opcode::And16mr:
        case Opcode::And32mr:
        case Opcode::And64mr:
        case Opcode::Or8mr:
        case Opcode::Or16mr:
        case Opcode::Or32mr:
        case Opcode::Or64mr:
        case Opcode::Xor8mr:
        case Opcode::Xor16mr:
        case Opcode::Xor32mr:
        case Opcode::Xor64mr:
        case Opcode::Cmp8mr:
        case Opcode::Cmp16mr:
        case Opcode::Cmp32mr:
        case Opcode::Cmp64mr:
            return mr;

        case Opcode::Mov8mi:
        case Opcode::Mov16mi:
        case Opcode::Mov32mi:
        case Opcode::Mov64mi:
        case Opcode::Add8mi:
        case Opcode::Add16mi:
        case Opcode::Add32mi:
        case Opcode::Add64mi:
        case Opcode::Sub8mi:
        case Opcode::Sub16mi:
        case Opcode::Sub32mi:
        case Opcode::Sub64mi:
        case Opcode::And8mi:
        case Opcode::And16mi:
        case Opcode::And32mi:
        case Opcode::And64mi:
        case Opcode::Or8mi:
        case Opcode::Or16mi:
        case Opcode::Or32mi:
        case Opcode::Or64mi:
        case Opcode::Xor8mi:
        case Opcode::Xor16mi:
        case Opcode::Xor32mi:
        case Opcode::Xor64mi:
            return mi;

        case Opcode::Shl8rc:
        case Opcode::Shl16rc:
        case Opcode::Shl32rc:
        case Opcode::Shl64rc:
        case Opcode::Shr8rc:
        case Opcode::Shr16rc:
        case Opcode::Shr32rc:
        case Opcode::Shr64rc:
        case Opcode::Sar8rc:
        case Opcode::Sar16rc:
        case Opcode::Sar32rc:
        case Opcode::Sar64rc:
            return r;

        case Opcode::Jmp:
        case Opcode::Je:
        case Opcode::Jne:
        case Opcode::Jl:
        case Opcode::Jle:
        case Opcode::Jg:
        case Opcode::Jge:
        case Opcode::Call:
        case Opcode::Ret:
            return empty;
    }
    return empty;
}

std::vector<Operand> InstructionInfo::getImplicitOperands()
{
    std::vector<Operand> implicit;

    switch (opcode_)
    {
        case Opcode::Imul8r:
            implicit.emplace_back(Register(RegisterKind::Al),
                                  OperandRole::DstSrc);
            implicit.emplace_back(Register(RegisterKind::Ah),
                                  OperandRole::Dst);
            break;
        case Opcode::Imul16r:
            implicit.emplace_back(Register(RegisterKind::Ax),
                                  OperandRole::DstSrc);
            implicit.emplace_back(Register(RegisterKind::Dx),
                                  OperandRole::Dst);
            break;
        case Opcode::Imul32r:
            implicit.emplace_back(Register(RegisterKind::Eax),
                                  OperandRole::DstSrc);
            implicit.emplace_back(Register(RegisterKind::Edx),
                                  OperandRole::Dst);
            break;
        case Opcode::Imul64r:
            implicit.emplace_back(Register(RegisterKind::Rax),
                                  OperandRole::DstSrc);
            implicit.emplace_back(Register(RegisterKind::Rdx),
                                  OperandRole::Dst);
            break;

        case Opcode::Div8r:
        case Opcode::Idiv8r:
            implicit.emplace_back(Register(RegisterKind::Al),
                                  OperandRole::DstSrc);
            implicit.emplace_back(Register(RegisterKind::Ah),
                                  OperandRole::DstSrc);
            break;
        case Opcode::Div16r:
        case Opcode::Idiv16r:
            implicit.emplace_back(Register(RegisterKind::Ax),
                                  OperandRole::DstSrc);
            implicit.emplace_back(Register(RegisterKind::Dx),
                                  OperandRole::DstSrc);
            break;
        case Opcode::Div32r:
        case Opcode::Idiv32r:
            implicit.emplace_back(Register(RegisterKind::Eax),
                                  OperandRole::DstSrc);
            implicit.emplace_back(Register(RegisterKind::Edx),
                                  OperandRole::DstSrc);
            break;
        case Opcode::Div64r:
        case Opcode::Idiv64r:
            implicit.emplace_back(Register(RegisterKind::Rax),
                                  OperandRole::DstSrc);
            implicit.emplace_back(Register(RegisterKind::Rdx),
                                  OperandRole::DstSrc);
            break;

        case Opcode::Shl8rc:
        case Opcode::Shl16rc:
        case Opcode::Shl32rc:
        case Opcode::Shl64rc:
        case Opcode::Shr8rc:
        case Opcode::Shr16rc:
        case Opcode::Shr32rc:
        case Opcode::Shr64rc:
        case Opcode::Sar8rc:
        case Opcode::Sar16rc:
        case Opcode::Sar32rc:
        case Opcode::Sar64rc:
            implicit.emplace_back(Register(RegisterKind::Cl),
                                  OperandRole::Src);
            break;

        case Opcode::Call:
        case Opcode::Ret:
            implicit.emplace_back(Register(RegisterKind::Rsp),
                                  OperandRole::DstSrc);
            break;

        default:
            break;
    }

    return implicit;
}

}  // namespace umbrella::x86