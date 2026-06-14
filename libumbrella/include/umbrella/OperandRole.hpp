#pragma once

#include <cstdint>

namespace umbrella {

enum class OperandRole : std::uint8_t
{
    Dst,
    Src,
    DstSrc,
};

}  // namespace umbrella
