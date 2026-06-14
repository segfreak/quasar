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
    TargetDescription(std::size_t ptr_size, CodeModel code_model)
        : ptr_size_(ptr_size), code_model_(code_model)
    {
    }

    std::size_t              getPointerSize() const { return ptr_size_; }
    CodeModel                getCodeModel() const { return code_model_; }

    static TargetDescription create(std::size_t ptr_size,
                                    CodeModel   code_model)
    { return TargetDescription{ptr_size, code_model}; }

    static TargetDescription host()
    { return TargetDescription::create(sizeof(void*), CodeModel::PIC); }

   private:
    std::size_t ptr_size_;
    CodeModel   code_model_;
};

}  // namespace umbrella
