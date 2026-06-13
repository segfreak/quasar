#pragma once

#include "TargetDescription.hpp"

namespace umbrella
{

struct Context
{
    const TargetDescription& getTargetDescription() const
    { return tdesc_; }

   private:
    TargetDescription tdesc_;
};

}  // namespace umbrella
