#pragma once

#include <cstddef>
#include <cstdint>

namespace umbrella
{

enum class CodeModel : std::uint8_t
{
    Static,
    PIC,
};

struct TargetDescription
{
    std::size_t getPointerSize() const { return ptr_size_; }
    CodeModel   getCodeModel() const { return code_model_; }

   private:
    std::size_t ptr_size_;
    CodeModel   code_model_;
};

}  // namespace umbrella
