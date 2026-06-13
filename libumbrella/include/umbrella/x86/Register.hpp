#pragma once

#include <cstdint>

namespace umbrella::x86
{

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
    Register(RegisterKind kind) : kind_(kind) {}

    RegisterKind getKind() const { return kind_; }
    std::uint8_t getId() const;
    std::size_t  getSize() const;

    bool         isCallerSaved() const;
    bool         requiresRexPrefix() const;

   private:
    RegisterKind kind_;
};

}  // namespace umbrella::x86