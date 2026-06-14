#pragma once

#include <cassert>
#include <cstdint>
#include <memory>

#include "Context.hpp"

namespace umbrella {

enum class TypeKind : std::uint8_t
{
    Void,

    Int8,
    Int16,
    Int32,
    Int64,
    Float32,
    Float64,

    Pointer,
};

struct Type
{
    Type() = delete;
    Type(TypeKind kind) : kind_(kind) {}

    TypeKind     getKind() const { return kind_; }
    std::uint8_t getId() const
    { return static_cast<std::uint8_t>(getKind()); }

    bool isPointer() const { return getKind() == TypeKind::Pointer; }
    bool isInteger() const
    {
        switch (getKind()) {
            case TypeKind::Int8:
            case TypeKind::Int16:
            case TypeKind::Int32:
            case TypeKind::Int64:
                return true;
            default:
                return false;
        }
    }

    bool isFloat() const
    {
        switch (getKind()) {
            case TypeKind::Float32:
            case TypeKind::Float64:
                return true;
            default:
                return false;
        }
    }

    bool isScalar() const
    { return isInteger() || isFloat() || isPointer(); }

    std::size_t getSize(std::unique_ptr<Context>& ctx) const
    {
        switch (getKind()) {
            case TypeKind::Int8:
                return 1;
            case TypeKind::Int16:
                return 2;
            case TypeKind::Int32:
                return 4;
            case TypeKind::Int64:
                return 8;
            case TypeKind::Float32:
                return 4;
            case TypeKind::Float64:
                return 8;
            case TypeKind::Pointer:
                if (ctx) {
                    return ctx->getTargetDescription().getPointerSize();
                }
                return 0;
            case TypeKind::Void:
                return 0;
            default:
                assert(false && "unhandled case");
        }
    }

    bool operator==(const Type& other) const
    { return getKind() == other.getKind(); }

    bool operator<(const Type& other) const
    { return getKind() < other.getKind(); }

   private:
    TypeKind kind_;
};

}  // namespace umbrella
