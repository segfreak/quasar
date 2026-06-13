#pragma once

#include <cstddef>

namespace umbrella
{

struct Register
{
    Register() = delete;
    Register(std::size_t ident) : id_(ident) {}

    std::size_t getId() const { return id_; }

    bool        operator==(const Register& other) const
    { return getId() == other.getId(); }
    bool operator<(const Register& other) const
    { return getId() < other.getId(); }

   private:
    std::size_t id_;
};

}  // namespace umbrella
