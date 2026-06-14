#include <cstdint>
#include <limits>
#include <umbrella/x86/Register.hpp>

#include "umbrella/Logging.hpp"

namespace umbrella::x86
{

std::uint8_t Register::getPhysicalId() const
{
    if (isVirtual()) { return std::numeric_limits<std::uint8_t>::max(); }

    if (!getPhysical().has_value())
    {
        errs("x86::Register::getPhysicalId()")
            << "bogus state: isVirtual() is false, but getPhysical() is "
               "std::nullopt\n";
        return std::numeric_limits<std::uint8_t>::max();
    }

    switch (getPhysical().value())
    {
        case RegisterKind::Al:
        case RegisterKind::Ax:
        case RegisterKind::Eax:
        case RegisterKind::Rax:
            return 0;
        case RegisterKind::Cl:
        case RegisterKind::Cx:
        case RegisterKind::Ecx:
        case RegisterKind::Rcx:
            return 1;
        case RegisterKind::Dl:
        case RegisterKind::Dx:
        case RegisterKind::Edx:
        case RegisterKind::Rdx:
            return 2;
        case RegisterKind::Bl:
        case RegisterKind::Bx:
        case RegisterKind::Ebx:
        case RegisterKind::Rbx:
            return 3;

        case RegisterKind::Ah:
            return 0;
        case RegisterKind::Ch:
            return 1;
        case RegisterKind::Dh:
            return 2;
        case RegisterKind::Bh:
            return 3;

        case RegisterKind::Sp:
        case RegisterKind::Esp:
        case RegisterKind::Rsp:
            return 4;
        case RegisterKind::Bp:
        case RegisterKind::Ebp:
        case RegisterKind::Rbp:
            return 5;
        case RegisterKind::Si:
        case RegisterKind::Esi:
        case RegisterKind::Rsi:
            return 6;
        case RegisterKind::Di:
        case RegisterKind::Edi:
        case RegisterKind::Rdi:
            return 7;

        case RegisterKind::R8w:
        case RegisterKind::R8d:
        case RegisterKind::R8:
            return 8;
        case RegisterKind::R9w:
        case RegisterKind::R9d:
        case RegisterKind::R9:
            return 9;
        case RegisterKind::R10w:
        case RegisterKind::R10d:
        case RegisterKind::R10:
            return 10;
        case RegisterKind::R11w:
        case RegisterKind::R11d:
        case RegisterKind::R11:
            return 11;
        case RegisterKind::R12w:
        case RegisterKind::R12d:
        case RegisterKind::R12:
            return 12;
        case RegisterKind::R13w:
        case RegisterKind::R13d:
        case RegisterKind::R13:
            return 13;
        case RegisterKind::R14w:
        case RegisterKind::R14d:
        case RegisterKind::R14:
            return 14;
        case RegisterKind::R15w:
        case RegisterKind::R15d:
        case RegisterKind::R15:
            return 15;
    }
    return std::numeric_limits<std::uint32_t>::max();
}

bool Register::requiresRexPrefix() const
{
    if (isVirtual())
    {
        // sanity: virtual registers does not requires rex prefix
        return false;
    }

    if (!getPhysical().has_value())
    {
        errs("x86::Register::requiresRexPrefix()")
            << "bogus state: isVirtual() is false, but getPhysical() is "
               "std::nullopt\n";
        return false;
    }

    switch (getPhysical().value())
    {
        case RegisterKind::R8w:
        case RegisterKind::R9w:
        case RegisterKind::R10w:
        case RegisterKind::R11w:
        case RegisterKind::R12w:
        case RegisterKind::R13w:
        case RegisterKind::R14w:
        case RegisterKind::R15w:

        case RegisterKind::R8d:
        case RegisterKind::R9d:
        case RegisterKind::R10d:
        case RegisterKind::R11d:
        case RegisterKind::R12d:
        case RegisterKind::R13d:
        case RegisterKind::R14d:
        case RegisterKind::R15d:

        case RegisterKind::R8:
        case RegisterKind::R9:
        case RegisterKind::R10:
        case RegisterKind::R11:
        case RegisterKind::R12:
        case RegisterKind::R13:
        case RegisterKind::R14:
        case RegisterKind::R15:
            return true;

        default:
            return false;
    }
}

std::size_t Register::getSize(std::unique_ptr<Context>& ctx) const
{
    if (isVirtual()) { return getVirtual()->getType().getSize(ctx); }

    if (!getPhysical().has_value())
    {
        errs("x86::Register::getSize(ctx)")
            << "bogus state: isVirtual() is false, but getPhysical() is "
               "std::nullopt\n";
        return 0;
    }

    switch (getPhysical().value())
    {
        // 8-bit
        case RegisterKind::Al:
        case RegisterKind::Bl:
        case RegisterKind::Cl:
        case RegisterKind::Dl:
        case RegisterKind::Ah:
        case RegisterKind::Bh:
        case RegisterKind::Ch:
        case RegisterKind::Dh:
            return 1;

        // 16-bit
        case RegisterKind::Ax:
        case RegisterKind::Bx:
        case RegisterKind::Cx:
        case RegisterKind::Dx:
        case RegisterKind::Sp:
        case RegisterKind::Bp:
        case RegisterKind::Si:
        case RegisterKind::Di:
        case RegisterKind::R8w:
        case RegisterKind::R9w:
        case RegisterKind::R10w:
        case RegisterKind::R11w:
        case RegisterKind::R12w:
        case RegisterKind::R13w:
        case RegisterKind::R14w:
        case RegisterKind::R15w:
            return 2;

        // 32-bit
        case RegisterKind::Eax:
        case RegisterKind::Ebx:
        case RegisterKind::Ecx:
        case RegisterKind::Edx:
        case RegisterKind::Esp:
        case RegisterKind::Ebp:
        case RegisterKind::Esi:
        case RegisterKind::Edi:
        case RegisterKind::R8d:
        case RegisterKind::R9d:
        case RegisterKind::R10d:
        case RegisterKind::R11d:
        case RegisterKind::R12d:
        case RegisterKind::R13d:
        case RegisterKind::R14d:
        case RegisterKind::R15d:
            return 4;

        // 64-bit
        case RegisterKind::Rax:
        case RegisterKind::Rbx:
        case RegisterKind::Rcx:
        case RegisterKind::Rdx:
        case RegisterKind::Rsp:
        case RegisterKind::Rbp:
        case RegisterKind::Rsi:
        case RegisterKind::Rdi:
        case RegisterKind::R8:
        case RegisterKind::R9:
        case RegisterKind::R10:
        case RegisterKind::R11:
        case RegisterKind::R12:
        case RegisterKind::R13:
        case RegisterKind::R14:
        case RegisterKind::R15:
            return 8;
    }

    return 0;
}

bool Register::isCallerSaved() const
{
    if (isVirtual())
    {
        // sanity: virtual registers does not requires rex prefix
        return false;
    }

    if (!getPhysical().has_value())
    {
        errs("x86::Register::isCallerSaved()")
            << "bogus state: isVirtual() is false, but getPhysical() is "
               "std::nullopt\n";
        return false;
    }

    switch (getPhysical().value())
    {
        case RegisterKind::Rax:
        case RegisterKind::Rcx:
        case RegisterKind::Rdx:
        case RegisterKind::Rsi:
        case RegisterKind::Rdi:

        case RegisterKind::R8:
        case RegisterKind::R9:
        case RegisterKind::R10:
        case RegisterKind::R11:

        case RegisterKind::Eax:
        case RegisterKind::Ecx:
        case RegisterKind::Edx:
        case RegisterKind::Esi:
        case RegisterKind::Edi:
        case RegisterKind::R8d:
        case RegisterKind::R9d:
        case RegisterKind::R10d:
        case RegisterKind::R11d:

        case RegisterKind::Ax:
        case RegisterKind::Cx:
        case RegisterKind::Dx:
        case RegisterKind::Si:
        case RegisterKind::Di:
        case RegisterKind::R8w:
        case RegisterKind::R9w:
        case RegisterKind::R10w:
        case RegisterKind::R11w:

        case RegisterKind::Al:
        case RegisterKind::Cl:
        case RegisterKind::Dl:
            return true;

        default:
            return false;
    }
}

}  // namespace umbrella::x86
