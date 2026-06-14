#pragma once

#include <memory>

#include "TargetDescription.hpp"

namespace umbrella {

struct Context
{
    Context() : tdesc_(TargetDescription::host()) {}

    TargetDescription&       getTargetDescription() { return tdesc_; }
    const TargetDescription& getTargetDescription() const
    { return tdesc_; }

    static std::unique_ptr<Context>& get()
    {
        thread_local std::unique_ptr<Context> instance{};
        return instance;
    }

   private:
    TargetDescription tdesc_;
};

}  // namespace umbrella
