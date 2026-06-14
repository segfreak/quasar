#pragma once

#include <unordered_map>

#include "umbrella/Function.hpp"
#include "umbrella/VirtualRegister.hpp"

namespace umbrella {

struct LiveInterval
{
    VirtualRegister vreg;
    std::size_t     start =
        -1;  // index of instruction where the vreg is defined
    std::size_t end =
        0;   // index of instruction where the vreg is last used

    bool intersects(const LiveInterval& other) const
    { return (start <= other.end && other.start <= end); }
};

class LiveIntervalBuilder
{
   public:
    static std::unordered_map<VirtualRegister, LiveInterval> build(
        const Function& function)
    {
        std::unordered_map<VirtualRegister, LiveInterval> intervals;

        for (const auto& arg : function.getArguments()) {
            intervals[arg] =
                LiveInterval{.vreg = arg, .start = 0, .end = 0};
        }

        std::size_t globalInstructionIndex = 0;

        for (const auto& block : function.getBlocks()) {
            for (const auto& instr : block.getInstructions()) {
                for (const auto& operand : instr.getOperands()) {
                    if (operand.isRegister()) {
                        VirtualRegister reg =
                            operand.getRegister().value();

                        if (!intervals.contains(reg)) {
                            intervals[reg] = LiveInterval{
                                .vreg  = reg,
                                .start = globalInstructionIndex,
                                .end   = globalInstructionIndex};
                        } else {
                            intervals[reg].end = globalInstructionIndex;
                        }
                    }
                }

                globalInstructionIndex += 2;
            }
        }

        return intervals;
    }
};

}  // namespace umbrella
