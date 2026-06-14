#pragma once

#include <span>
#include <variant>

namespace umbrella
{

template <typename RegisterT, typename SpillT>
struct RegAlloc
{
    virtual std::variant<RegisterT, SpillT> alloc()                  = 0;
    virtual void                            release(RegisterT reg)   = 0;
    virtual void                            release(SpillT spill)    = 0;
    virtual std::span<const RegisterT> getAvailableRegisters() const = 0;
    virtual ~RegAlloc() = default;
};

}  // namespace umbrella