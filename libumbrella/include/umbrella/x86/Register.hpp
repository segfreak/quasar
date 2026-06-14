#pragma once

#include <cstdint>
#include <optional>
#include <variant>

#include "../Context.hpp"
#include "../VirtualRegister.hpp"

namespace umbrella::x86 {

enum class RegisterKind : std::uint8_t
{
    // 8-bit registers
    Al,
    Bl,
    Cl,
    Dl,
    Ah,
    Bh,
    Ch,
    Dh,

    // 16-bit registers
    Ax,
    Bx,
    Cx,
    Dx,
    Sp,
    Bp,
    Si,
    Di,
    R8w,
    R9w,
    R10w,
    R11w,
    R12w,
    R13w,
    R14w,
    R15w,

    // 32-bit registers
    Eax,
    Ebx,
    Ecx,
    Edx,
    Esp,
    Ebp,
    Esi,
    Edi,
    R8d,
    R9d,
    R10d,
    R11d,
    R12d,
    R13d,
    R14d,
    R15d,

    // 64-bit registers
    Rax,
    Rbx,
    Rcx,
    Rdx,
    Rsp,
    Rbp,
    Rsi,
    Rdi,
    R8,
    R9,
    R10,
    R11,
    R12,
    R13,
    R14,
    R15,
};

struct Register
{
    Register(RegisterKind kind) : value_(kind) {}
    Register(VirtualRegister vreg) : value_(vreg) {}

    template <typename T>
    constexpr bool is() const
    { return std::holds_alternative<T>(value_); }

    template <typename T>
    std::optional<T> get() const
    {
        if (is<T>()) { return std::get<T>(value_); }
        return std::nullopt;
    }

    std::optional<RegisterKind> getPhysical() const
    { return get<RegisterKind>(); }

    std::optional<VirtualRegister> getVirtual() const
    { return get<VirtualRegister>(); }

    bool         isVirtual() const { return is<VirtualRegister>(); }
    bool         isPhysical() const { return is<RegisterKind>(); }

    std::uint8_t getPhysicalId() const;
    std::size_t  getSize(std::unique_ptr<Context>& ctx) const;

    bool         isCallerSaved() const;
    bool         requiresRexPrefix() const;

   private:
    std::variant<std::monostate, VirtualRegister, RegisterKind> value_;
};

}  // namespace umbrella::x86