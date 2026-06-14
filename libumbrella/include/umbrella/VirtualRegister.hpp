#pragma once

#include <cstddef>
#include <tuple>

#include "Type.hpp"

namespace umbrella {

struct VirtualRegister
{
    VirtualRegister(std::size_t id, Type type) : id_(id), type_(type) {}

    // copy constructor
    VirtualRegister(const VirtualRegister& other)
        : id_(other.getId()), type_(other.getType())
    {
    }

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

namespace std {
template <>
struct hash<umbrella::VirtualRegister>
{
    std::size_t operator()(
        const umbrella::VirtualRegister& reg) const noexcept
    {
        std::size_t h1 = std::hash<std::size_t>{}(reg.getId());
        std::size_t h2 = std::hash<std::size_t>{}(reg.getType().getId());
        return h1 ^ (h2 + 0x9e3779b9 + (h1 << 6) + (h1 >> 2));
    }
};
}  // namespace std