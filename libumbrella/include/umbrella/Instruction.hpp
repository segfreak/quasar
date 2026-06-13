#pragma once

#include <cstdint>
#include <ranges>
#include <span>
#include <vector>

#include "Register.hpp"

namespace umbrella
{

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

enum class OperandRole : std::uint8_t
{
    Dst,
    Src,
    DstSrc,
};

struct Operand
{
    Operand() = delete;
    Operand(Register reg, OperandRole role) : reg_(reg), role_(role) {}

    Register    getRegister() const { return reg_; }
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
    Register    reg_;
    OperandRole role_;
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
        return operands_ |
               std::views::filter([](const Operand& op)
                                  { return op.isDestination(); });
    }

    /// get the source operands
    auto getSources() const
    {
        return operands_ | std::views::filter([](const Operand& op)
                                              { return op.isSource(); });
    }

    // check that the instructions are legal
    bool verify() const;

   private:
    Opcode               opcode_;
    std::vector<Operand> operands_;
};

}  // namespace umbrella
