#pragma once

#include <cstdint>
#include <optional>
#include <utility>
#include <variant>

#include "../Instruction.hpp"
#include "Register.hpp"

namespace umbrella::x86
{

enum class OperandKind : std::uint8_t
{
    Register,
    Immediate,
    Memory,
};

using Immediate = std::uint64_t;

enum class Scale : std::uint8_t
{
    One   = 1,
    Two   = 2,
    Four  = 4,
    Eight = 8
};

struct Memory
{
    bool hasBase() const { return base_.has_value(); }
    bool hasIndex() const { return index_.has_value(); }
    bool isAbsolute() const
    { return !base_.has_value() && !index_.has_value(); }

    const std::optional<Register>& getBase() const { return base_; }
    const std::optional<Register>& getIndex() const { return index_; }
    std::uint64_t getDisplacement() const { return displacement_; }
    Scale         getScale() const { return scale_; }

    bool          hasSIB() const { return index_.has_value(); }

   private:
    std::optional<Register> base_;
    std::optional<Register> index_;
    Scale                   scale_        = Scale::One;
    std::int64_t            displacement_ = 0;
};
struct Operand
{
    Operand(Register reg, OperandRole role) : value_(reg), role_(role) {}
    Operand(Immediate imm, OperandRole role) : value_(imm), role_(role) {}
    Operand(Memory mem, OperandRole role) : value_(mem), role_(role) {}

    template <typename T>
    constexpr bool is() const
    { return std::holds_alternative<T>(value_); }

    template <typename T>
    std::optional<T> get() const
    {
        if (is<T>()) { return std::get<T>(value_); }
        return std::nullopt;
    }

    bool isMonostate() const { return is<std::monostate>(); }
    bool isRegister() const { return is<Register>(); }
    bool isImmediate() const { return is<Immediate>(); }
    bool isMemory() const { return is<Memory>(); }

    std::optional<Register> getRegister() const { return get<Register>(); }
    std::optional<Immediate> getImmediate() const
    { return get<Immediate>(); }
    std::optional<Memory>      getMemory() const { return get<Memory>(); }

    std::optional<OperandKind> getKind() const
    {
        return std::visit(
            [](auto&& v)
            {
                using T = std::decay_t<decltype(v)>;

                if constexpr (std::is_same_v<T, Register>)
                {
                    return OperandKind::Register;
                }
                if constexpr (std::is_same_v<T, Immediate>)
                {
                    return OperandKind::Immediate;
                }
                if constexpr (std::is_same_v<T, Memory>)
                {
                    return OperandKind::Memory;
                }
                if constexpr (std::is_same_v<T, std::monostate>)
                {
                    std::unreachable();
                    return static_cast<OperandKind>(-1);
                }
            },
            value_);
    }

    OperandRole getRole() const { return role_; }

    // is read and write
    bool        isReadAndWrite() const
    { return getRole() == OperandRole::DstSrc; }

    // is read
    bool isReadOnly() const { return getRole() == OperandRole::Src; }
    // is write
    bool isWriteOnly() const { return getRole() == OperandRole::Dst; }

    // is read (or read and write)
    bool isRead() const { return isReadOnly() || isReadAndWrite(); }
    // is write (or read and write)
    bool isWrite() const { return isWriteOnly() || isReadAndWrite(); }

   private:
    std::variant<std::monostate, Register, Immediate, Memory> value_;
    OperandRole                                               role_;
};

}  // namespace umbrella::x86