#pragma once

#include <cstddef>
#include <tuple>

#include "Type.hpp"

namespace umbrella
{

struct VirtualRegister
{
    VirtualRegister() = delete;
    VirtualRegister(std::size_t id, Type type) : id_(id), type_(type) {}

    std::size_t getId() const { return id_; }
    Type        getType() const { return type_; }

    bool        operator==(const VirtualRegister& other) const
    { return std::tie(id_, type_) == std::tie(other.id_, other.type_); }

    bool operator<(const VirtualRegister& other) const
    { return std::tie(id_, type_) < std::tie(other.id_, other.type_); }

   private:
    std::size_t id_;
    Type        type_;
};

}  // namespace umbrella
