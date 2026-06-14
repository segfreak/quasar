#pragma once

#include <cstdint>
#include <ranges>
#include <span>
#include <variant>
#include <vector>

#include "OperandRole.hpp"
#include "VirtualRegister.hpp"

namespace umbrella {

using Immediate = std::uint64_t;

enum class Opcode : std::uint8_t
{
    // mov dst, src
    Mov,
    // add dst, src, src2
    Add,
    // sub dst, src, src2
    Sub,
    // ret src
    Ret,
};

struct Operand
{
    Operand() = delete;
    Operand(VirtualRegister reg, OperandRole role)
        : value_(reg), role_(role)
    {
    }
    Operand(Immediate imm, OperandRole role) : value_(imm), role_(role) {}
    Operand(std::variant<std::monostate, VirtualRegister, Immediate> var,
            OperandRole                                              role)
        : value_(var), role_(role)
    {
    }

    template <typename T>
    constexpr bool is() const
    { return std::holds_alternative<T>(value_); }

    template <typename T>
    std::optional<T> get() const
    {
        if (is<T>()) { return std::get<T>(value_); }
        return std::nullopt;
    }

    bool isRegister() const { return is<VirtualRegister>(); }
    bool isImmediate() const { return is<Immediate>(); }

    std::optional<VirtualRegister> getRegister() const
    { return get<VirtualRegister>(); }
    std::optional<Immediate> getImmediate() const
    { return get<Immediate>(); }

    OperandRole getRole() const { return role_; }

    bool        isDestinationOnly() const
    { return getRole() == OperandRole::Dst; }
    bool isSourceOnly() const { return getRole() == OperandRole::Src; }
    bool isDestinationAndSource() const
    { return getRole() == OperandRole::DstSrc; }

    bool isDestination() const
    { return isDestinationOnly() || isDestinationAndSource(); }
    bool isSource() const
    { return isSourceOnly() || isDestinationAndSource(); }

   private:
    std::variant<std::monostate, VirtualRegister, Immediate> value_;
    OperandRole                                              role_;
};

// gets a expected operand roles for opcode
std::vector<OperandRole> getExpectedOperandRolesFor(Opcode opcode);

struct Instruction
{
    Instruction() = delete;
    Instruction(Opcode opcode, std::vector<Operand> operands)
        : opcode_(opcode), operands_(std::move(operands))
    {
    }

    // copy constructor
    Instruction(const Instruction& oth)
        : opcode_(oth.opcode_), operands_(oth.operands_)
    {
    }

    // move constructor
    Instruction(Instruction&& oth) noexcept
        : opcode_(oth.opcode_), operands_(std::move(oth.operands_))
    {
    }

    Opcode                   getOpcode() const { return opcode_; }
    std::span<const Operand> getOperands() const { return operands_; }

    /// get the destination operands
    auto                     getDestinations() const
    {
        return operands_ | std::views::filter([](const Operand& op) {
                   return op.isDestination();
               });
    }

    /// get the source operands
    auto getSources() const
    {
        return operands_ | std::views::filter([](const Operand& op) {
                   return op.isSource();
               });
    }

    // check that the instructions are legal
    bool verify() const;

   private:
    Opcode               opcode_;
    std::vector<Operand> operands_;
};

}  // namespace umbrella
