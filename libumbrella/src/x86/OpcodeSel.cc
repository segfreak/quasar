#include "umbrella/x86/OpcodeSel.hpp"

#include <cstdint>

#include "umbrella/support/Logging.hpp"
#include "umbrella/x86/InstructionSet.hpp"
#include "umbrella/x86/Operand.hpp"
#include "umbrella/x86/ToString.hpp"

namespace umbrella::x86 {

std::uint8_t OpcodeSel::extractBitWidth(const Operand& op)
{
    if (op.isRegister()) {
        auto reg = op.getRegister().value();
        return reg.getSize(Context::get()) * 8;
    }
    return 64;
}

std::optional<X86Opcode> OpcodeSel::select(
    X86Mnemonic mnemonic, const std::vector<Operand>& operands)
{
    dbgs("OpcodeSel") << "selection for mnemonic: " << toString(mnemonic)
                      << "\n";

    if (operands.empty()) {
        if (mnemonic == X86Mnemonic::Jmp) { return X86Opcode::Jmp; }
        if (mnemonic == X86Mnemonic::Ret) { return X86Opcode::Ret; }
        if (mnemonic == X86Mnemonic::Call) { return X86Opcode::Call; }
        return std::nullopt;
    }

    std::uint8_t bitWidth = extractBitWidth(operands[0]);
    auto         dstKind  = operands[0].getKind();
    if (!dstKind) { return std::nullopt; }

    if (operands.size() == 1) {
        if (mnemonic == X86Mnemonic::Je) { return X86Opcode::Je; }
        if (mnemonic == X86Mnemonic::Jne) { return X86Opcode::Jne; }
        if (mnemonic == X86Mnemonic::Jl) { return X86Opcode::Jl; }
        if (mnemonic == X86Mnemonic::Jle) { return X86Opcode::Jle; }
        if (mnemonic == X86Mnemonic::Jg) { return X86Opcode::Jg; }
        if (mnemonic == X86Mnemonic::Jge) { return X86Opcode::Jge; }
        if (mnemonic == X86Mnemonic::Sete) { return X86Opcode::Sete; }
        if (mnemonic == X86Mnemonic::Setne) { return X86Opcode::Setne; }
        if (mnemonic == X86Mnemonic::Setl) { return X86Opcode::Setl; }
        if (mnemonic == X86Mnemonic::Setle) { return X86Opcode::Setle; }
        if (mnemonic == X86Mnemonic::Setg) { return X86Opcode::Setg; }
        if (mnemonic == X86Mnemonic::Setge) { return X86Opcode::Setge; }

        return selectUnary(mnemonic, bitWidth, *dstKind);
    }

    if (operands.size() == 2) {
        auto srcKind = operands[1].getKind();
        if (!srcKind) { return std::nullopt; }

        if (mnemonic == X86Mnemonic::Movzx ||
            mnemonic == X86Mnemonic::Movsx) {
            std::uint8_t dstWidth = extractBitWidth(operands[0]);
            std::uint8_t srcWidth = extractBitWidth(operands[1]);
            return selectExt(mnemonic, srcWidth, dstWidth);
        }

        if (mnemonic == X86Mnemonic::Shl || mnemonic == X86Mnemonic::Shr ||
            mnemonic == X86Mnemonic::Sar) {
            return selectShift(mnemonic, bitWidth, *dstKind, *srcKind,
                               operands);
        }

        return selectBinary(mnemonic, bitWidth, *dstKind, *srcKind);
    }

    return std::nullopt;
}

std::optional<X86Opcode> OpcodeSel::selectBinary(X86Mnemonic  mnemonic,
                                                 std::uint8_t width,
                                                 OperandKind  dst,
                                                 OperandKind  src)
{
#define MAP_BIN(MnemName, OpPrefix)                                       \
    if (mnemonic == X86Mnemonic::MnemName) {                              \
        if (width == 8) {                                                 \
            if (dst == OperandKind::Register &&                           \
                src == OperandKind::Register)                             \
                return X86Opcode::OpPrefix##8rr;                          \
            if (dst == OperandKind::Register &&                           \
                src == OperandKind::Immediate)                            \
                return X86Opcode::OpPrefix##8ri;                          \
            if (dst == OperandKind::Register &&                           \
                src == OperandKind::Memory)                               \
                return X86Opcode::OpPrefix##8rm;                          \
            if (dst == OperandKind::Memory &&                             \
                src == OperandKind::Register)                             \
                return X86Opcode::OpPrefix##8mr;                          \
            if (dst == OperandKind::Memory &&                             \
                src == OperandKind::Immediate)                            \
                return X86Opcode::OpPrefix##8mi;                          \
        } else if (width == 16) {                                         \
            if (dst == OperandKind::Register &&                           \
                src == OperandKind::Register)                             \
                return X86Opcode::OpPrefix##16rr;                         \
            if (dst == OperandKind::Register &&                           \
                src == OperandKind::Immediate)                            \
                return X86Opcode::OpPrefix##16ri;                         \
            if (dst == OperandKind::Register &&                           \
                src == OperandKind::Memory)                               \
                return X86Opcode::OpPrefix##16rm;                         \
            if (dst == OperandKind::Memory &&                             \
                src == OperandKind::Register)                             \
                return X86Opcode::OpPrefix##16mr;                         \
            if (dst == OperandKind::Memory &&                             \
                src == OperandKind::Immediate)                            \
                return X86Opcode::OpPrefix##16mi;                         \
        } else if (width == 32) {                                         \
            if (dst == OperandKind::Register &&                           \
                src == OperandKind::Register)                             \
                return X86Opcode::OpPrefix##32rr;                         \
            if (dst == OperandKind::Register &&                           \
                src == OperandKind::Immediate)                            \
                return X86Opcode::OpPrefix##32ri;                         \
            if (dst == OperandKind::Register &&                           \
                src == OperandKind::Memory)                               \
                return X86Opcode::OpPrefix##32rm;                         \
            if (dst == OperandKind::Memory &&                             \
                src == OperandKind::Register)                             \
                return X86Opcode::OpPrefix##32mr;                         \
            if (dst == OperandKind::Memory &&                             \
                src == OperandKind::Immediate)                            \
                return X86Opcode::OpPrefix##32mi;                         \
        } else if (width == 64) {                                         \
            if (dst == OperandKind::Register &&                           \
                src == OperandKind::Register)                             \
                return X86Opcode::OpPrefix##64rr;                         \
            if (dst == OperandKind::Register &&                           \
                src == OperandKind::Immediate)                            \
                return X86Opcode::OpPrefix##64ri;                         \
            if (dst == OperandKind::Register &&                           \
                src == OperandKind::Memory)                               \
                return X86Opcode::OpPrefix##64rm;                         \
            if (dst == OperandKind::Memory &&                             \
                src == OperandKind::Register)                             \
                return X86Opcode::OpPrefix##64mr;                         \
            if (dst == OperandKind::Memory &&                             \
                src == OperandKind::Immediate)                            \
                return X86Opcode::OpPrefix##64mi;                         \
        }                                                                 \
    }

    MAP_BIN(Mov, Mov)
    MAP_BIN(Add, Add)
    MAP_BIN(Sub, Sub)
    MAP_BIN(And, And)
    MAP_BIN(Or, Or)
    MAP_BIN(Xor, Xor)

#undef MAP_BIN

    if (mnemonic == X86Mnemonic::Cmp) {
        if (width == 8) {
            if (dst == OperandKind::Register &&
                src == OperandKind::Register) {
                return X86Opcode::Cmp8rr;
            }
            if (dst == OperandKind::Register &&
                src == OperandKind::Immediate) {
                return X86Opcode::Cmp8ri;
            }
            if (dst == OperandKind::Register &&
                src == OperandKind::Memory) {
                return X86Opcode::Cmp8rm;
            }
            if (dst == OperandKind::Memory &&
                src == OperandKind::Register) {
                return X86Opcode::Cmp8mr;
            }
        } else if (width == 16) {
            if (dst == OperandKind::Register &&
                src == OperandKind::Register) {
                return X86Opcode::Cmp16rr;
            }
            if (dst == OperandKind::Register &&
                src == OperandKind::Immediate) {
                return X86Opcode::Cmp16ri;
            }
            if (dst == OperandKind::Register &&
                src == OperandKind::Memory) {
                return X86Opcode::Cmp16rm;
            }
            if (dst == OperandKind::Memory &&
                src == OperandKind::Register) {
                return X86Opcode::Cmp16mr;
            }
        } else if (width == 32) {
            if (dst == OperandKind::Register &&
                src == OperandKind::Register) {
                return X86Opcode::Cmp32rr;
            }
            if (dst == OperandKind::Register &&
                src == OperandKind::Immediate) {
                return X86Opcode::Cmp32ri;
            }
            if (dst == OperandKind::Register &&
                src == OperandKind::Memory) {
                return X86Opcode::Cmp32rm;
            }
            if (dst == OperandKind::Memory &&
                src == OperandKind::Register) {
                return X86Opcode::Cmp32mr;
            }
        } else if (width == 64) {
            if (dst == OperandKind::Register &&
                src == OperandKind::Register) {
                return X86Opcode::Cmp64rr;
            }
            if (dst == OperandKind::Register &&
                src == OperandKind::Immediate) {
                return X86Opcode::Cmp64ri;
            }
            if (dst == OperandKind::Register &&
                src == OperandKind::Memory) {
                return X86Opcode::Cmp64rm;
            }
            if (dst == OperandKind::Memory &&
                src == OperandKind::Register) {
                return X86Opcode::Cmp64mr;
            }
        }
    }

    if (mnemonic == X86Mnemonic::Lea) {
        if (dst == OperandKind::Register && src == OperandKind::Memory) {
            if (width == 16) { return X86Opcode::Lea16rm; }
            if (width == 32) { return X86Opcode::Lea32rm; }
            if (width == 64) { return X86Opcode::Lea64rm; }
        }
    }

    if (mnemonic == X86Mnemonic::Test && dst == OperandKind::Register &&
        src == OperandKind::Register) {
        if (width == 8) { return X86Opcode::Test8rr; }
        if (width == 16) { return X86Opcode::Test16rr; }
        if (width == 32) { return X86Opcode::Test32rr; }
        if (width == 64) { return X86Opcode::Test64rr; }
    }

    if (mnemonic == X86Mnemonic::Imul && dst == OperandKind::Register &&
        src == OperandKind::Register) {
        if (width == 8) { return X86Opcode::Imul8rr; }
        if (width == 16) { return X86Opcode::Imul16rr; }
        if (width == 32) { return X86Opcode::Imul32rr; }
        if (width == 64) { return X86Opcode::Imul64rr; }
    }

    return std::nullopt;
}

std::optional<X86Opcode> OpcodeSel::selectShift(
    X86Mnemonic mnemonic, std::uint8_t width, OperandKind dst,
    OperandKind src, const std::vector<Operand>& operands)
{
    errs("OpcodeSel") << "selectShift: mnemonic=" << toString(mnemonic)
                      << ", width=" << static_cast<int>(width)
                      << ", dst=" << toString(dst)
                      << ", src=" << toString(src) << "\n";

    bool isCl = false;
    if (src == OperandKind::Register) { isCl = true; }

#define MAP_SHIFT(MnemName, OpPrefix)                                     \
    if (mnemonic == X86Mnemonic::MnemName &&                              \
        dst == OperandKind::Register) {                                   \
        if (width == 8) {                                                 \
            if (src == OperandKind::Immediate)                            \
                return X86Opcode::OpPrefix##8ri;                          \
            if (src == OperandKind::Memory)                               \
                return X86Opcode::OpPrefix##8rm;                          \
            if (isCl) return X86Opcode::OpPrefix##8rc;                    \
        } else if (width == 16) {                                         \
            if (src == OperandKind::Immediate)                            \
                return X86Opcode::OpPrefix##16ri;                         \
            if (src == OperandKind::Memory)                               \
                return X86Opcode::OpPrefix##16rm;                         \
            if (isCl) return X86Opcode::OpPrefix##16rc;                   \
        } else if (width == 32) {                                         \
            if (src == OperandKind::Immediate)                            \
                return X86Opcode::OpPrefix##32ri;                         \
            if (src == OperandKind::Memory)                               \
                return X86Opcode::OpPrefix##32rm;                         \
            if (isCl) return X86Opcode::OpPrefix##32rc;                   \
        } else if (width == 64) {                                         \
            if (src == OperandKind::Immediate)                            \
                return X86Opcode::OpPrefix##64ri;                         \
            if (src == OperandKind::Memory)                               \
                return X86Opcode::OpPrefix##64rm;                         \
            if (isCl) return X86Opcode::OpPrefix##64rc;                   \
        }                                                                 \
    }

    MAP_SHIFT(Shl, Shl)
    MAP_SHIFT(Shr, Shr)
    MAP_SHIFT(Sar, Sar)

#undef MAP_SHIFT
    return std::nullopt;
}

std::optional<X86Opcode> OpcodeSel::selectUnary(X86Mnemonic  mnemonic,
                                                std::uint8_t width,
                                                OperandKind  dst)
{
    if (dst != OperandKind::Register) { return std::nullopt; }

    if (mnemonic == X86Mnemonic::Neg) {
        if (width == 8) { return X86Opcode::Neg8r; }
        if (width == 16) { return X86Opcode::Neg16r; }
        if (width == 32) { return X86Opcode::Neg32r; }
        if (width == 64) { return X86Opcode::Neg64r; }
    }
    if (mnemonic == X86Mnemonic::Not) {
        if (width == 8) { return X86Opcode::Not8r; }
        if (width == 16) { return X86Opcode::Not16r; }
        if (width == 32) { return X86Opcode::Not32r; }
        if (width == 64) { return X86Opcode::Not64r; }
    }
    if (mnemonic == X86Mnemonic::Div) {
        if (width == 8) { return X86Opcode::Div8r; }
        if (width == 16) { return X86Opcode::Div16r; }
        if (width == 32) { return X86Opcode::Div32r; }
        if (width == 64) { return X86Opcode::Div64r; }
    }
    if (mnemonic == X86Mnemonic::Idiv) {
        if (width == 8) { return X86Opcode::Idiv8r; }
        if (width == 16) { return X86Opcode::Idiv16r; }
        if (width == 32) { return X86Opcode::Idiv32r; }
        if (width == 64) { return X86Opcode::Idiv64r; }
    }
    if (mnemonic == X86Mnemonic::Imul) {
        if (width == 8) { return X86Opcode::Imul8r; }
        if (width == 16) { return X86Opcode::Imul16r; }
        if (width == 32) { return X86Opcode::Imul32r; }
        if (width == 64) { return X86Opcode::Imul64r; }
    }

    return std::nullopt;
}

std::optional<X86Opcode> OpcodeSel::selectExt(X86Mnemonic  mnemonic,
                                              std::uint8_t srcWidth,
                                              std::uint8_t dstWidth)
{
    if (mnemonic == X86Mnemonic::Movzx) {
        if (srcWidth == 8 && dstWidth == 16) {
            return X86Opcode::Movzx8_16;
        }
        if (srcWidth == 8 && dstWidth == 32) {
            return X86Opcode::Movzx8_32;
        }
        if (srcWidth == 8 && dstWidth == 64) {
            return X86Opcode::Movzx8_64;
        }
        if (srcWidth == 16 && dstWidth == 32) {
            return X86Opcode::Movzx16_32;
        }
        if (srcWidth == 16 && dstWidth == 64) {
            return X86Opcode::Movzx16_64;
        }
        if (srcWidth == 32 && dstWidth == 64) {
            return X86Opcode::Movzx32_64;
        }
    }
    if (mnemonic == X86Mnemonic::Movsx) {
        if (srcWidth == 8 && dstWidth == 16) {
            return X86Opcode::Movsx8_16;
        }
        if (srcWidth == 8 && dstWidth == 32) {
            return X86Opcode::Movsx8_32;
        }
        if (srcWidth == 8 && dstWidth == 64) {
            return X86Opcode::Movsx8_64;
        }
        if (srcWidth == 16 && dstWidth == 32) {
            return X86Opcode::Movsx16_32;
        }
        if (srcWidth == 16 && dstWidth == 64) {
            return X86Opcode::Movsx16_64;
        }
        if (srcWidth == 32 && dstWidth == 64) {
            return X86Opcode::Movsx32_64;
        }
    }
    return std::nullopt;
}

}  // namespace umbrella::x86