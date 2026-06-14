#pragma once

#include <span>
#include <variant>

namespace umbrella {

template <typename RegisterT, typename SpillT>
struct RegAllocTraits
{
    using register_type                                     = RegisterT;
    using spill_type                                        = SpillT;

    virtual std::variant<register_type, spill_type> alloc() = 0;
    virtual void                           release(register_type reg) = 0;
    virtual void                           release(spill_type spill)  = 0;
    virtual std::span<const register_type> getAvailableRegisters()
        const                 = 0;

    virtual ~RegAllocTraits() = default;
};

}  // namespace umbrella