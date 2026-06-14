#pragma once

#include <vector>

#include "InstructionSet.hpp"
#include "Operand.hpp"

namespace umbrella::x86 {

class OpcodeSel
{
   public:
    static std::optional<X86Opcode> select(
        X86Mnemonic mnemonic, const std::vector<Operand>& operands);

   private:
    static std::uint8_t             extractBitWidth(const Operand& op);

    static std::optional<X86Opcode> selectBinary(X86Mnemonic  mnemonic,
                                                 std::uint8_t width,
                                                 OperandKind  dst,
                                                 OperandKind  src);
    static std::optional<X86Opcode> selectShift(
        X86Mnemonic mnemonic, std::uint8_t width, OperandKind dst,
        OperandKind src, const std::vector<Operand>& operands);
    static std::optional<X86Opcode> selectUnary(X86Mnemonic  mnemonic,
                                                std::uint8_t width,
                                                OperandKind  dst);
    static std::optional<X86Opcode> selectExt(X86Mnemonic  mnemonic,
                                              std::uint8_t srcWidth,
                                              std::uint8_t dstWidth);
};

}  // namespace umbrella::x86