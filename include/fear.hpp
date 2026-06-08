#pragma once

#include <cstdint>
#include <stdexcept>
#include <string>
#include <string_view>
#include <vector>

#include "fear.h"

namespace fear
{

// Forward declarations
struct Module;
struct Function;
struct FunctionDef;

/**
 * @brief Supported types in the Fear IR.
 */
enum class Type
{
    Void,
    Bool,
    Int8,
    Int16,
    Int32,
    Int64,
    Float32,
    Float64,
    Pointer,
};

/**
 * @brief Linkage types for functions and symbols.
 */
enum class Linkage
{
    External,
    Internal,
    Weak,
};

/**
 * @brief Optimization levels for the compiler backend.
 */
enum class OptLevel
{
    None,
    Default,
    Full,
};

/**
 * @brief Calling conventions for function declarations.
 */
enum class CallConv
{
    C,
    SysV,
    MsAbi,
};

/**
 * @brief Code generation backends.
 */
enum class Backend
{
    Dummy,
    Cranelift,
    Llvm,
};

// Type aliases for compiler primitives
using ValueId    = FearValueId;
using BlockId    = FearBlockId;
using FuncId     = FearFuncId;

using RawFuncDef = FearFunctionDef;
using RawModule  = FearModule;

namespace detail
{
/**
 * @brief Converts C++ high-level enums to raw C API constants.
 */
inline FearType into(Type ty)
{
    switch (ty)
    {
        case Type::Void:
            return FearVoid;
        case Type::Bool:
            return FearBool;
        case Type::Int8:
            return FearInt8;
        case Type::Int16:
            return FearInt16;
        case Type::Int32:
            return FearInt32;
        case Type::Int64:
            return FearInt64;
        case Type::Float32:
            return FearFloat32;
        case Type::Float64:
            return FearFloat64;
        case Type::Pointer:
            return FearPointer;
    }
    throw std::runtime_error("Unknown Fear Type");
}

inline FearLinkage into(Linkage linkage)
{
    switch (linkage)
    {
        case Linkage::External:
            return FearLinkageExternal;
        case Linkage::Internal:
            return FearLinkageInternal;
        case Linkage::Weak:
            return FearLinkageWeak;
    }
    throw std::runtime_error("Unknown Fear Linkage");
}

inline FearOptLevel into(OptLevel level)
{
    switch (level)
    {
        case OptLevel::None:
            return FearOptNone;
        case OptLevel::Default:
            return FearOptDefault;
        case OptLevel::Full:
            return FearOptFull;
    }
    throw std::runtime_error("Unknown Fear OptLevel");
}

inline FearCallConv into(CallConv cc)
{
    switch (cc)
    {
        case CallConv::C:
            return FearCallConvC;
        case CallConv::SysV:
            return FearCallConvSysV;
        case CallConv::MsAbi:
            return FearCallConvMsAbi;
    }
    throw std::runtime_error("Unknown Fear CallConv");
}

inline FearBackend into(Backend backend)
{
    switch (backend)
    {
        case Backend::Dummy:
            return FearBackendDummy;
        case Backend::Cranelift:
            return FearBackendCranelift;
        case Backend::Llvm:
            return FearBackendLlvm;
    }
    throw std::runtime_error("Unknown Fear Backend");
}

/**
 * @brief Converts raw C API constants back to C++ high-level enums.
 */
inline Type from(FearType ty)
{
    switch (ty)
    {
        case FearVoid:
            return Type::Void;
        case FearBool:
            return Type::Bool;
        case FearInt8:
            return Type::Int8;
        case FearInt16:
            return Type::Int16;
        case FearInt32:
            return Type::Int32;
        case FearInt64:
            return Type::Int64;
        case FearFloat32:
            return Type::Float32;
        case FearFloat64:
            return Type::Float64;
        case FearPointer:
            return Type::Pointer;
    }
    throw std::runtime_error("Unknown FearType");
}

inline Linkage from(FearLinkage linkage)
{
    switch (linkage)
    {
        case FearLinkageExternal:
            return Linkage::External;
        case FearLinkageInternal:
            return Linkage::Internal;
        case FearLinkageWeak:
            return Linkage::Weak;
    }
    throw std::runtime_error("Unknown FearLinkage");
}

inline OptLevel from(FearOptLevel level)
{
    switch (level)
    {
        case FearOptNone:
            return OptLevel::None;
        case FearOptDefault:
            return OptLevel::Default;
        case FearOptFull:
            return OptLevel::Full;
    }
    throw std::runtime_error("Unknown FearOptLevel");
}

inline CallConv from(FearCallConv cc)
{
    switch (cc)
    {
        case FearCallConvC:
            return CallConv::C;
        case FearCallConvSysV:
            return CallConv::SysV;
        case FearCallConvMsAbi:
            return CallConv::MsAbi;
    }
    throw std::runtime_error("Unknown FearCallConv");
}

inline Backend from(FearBackend backend)
{
    switch (backend)
    {
        case FearBackendDummy:
            return Backend::Dummy;
        case FearBackendCranelift:
            return Backend::Cranelift;
        case FearBackendLlvm:
            return Backend::Llvm;
    }
    throw std::runtime_error("Unknown FearBackend");
}
}  // namespace detail

/**
 * @brief Selects the default host-native backend for target object
 * emission.
 */
inline Backend selectBackendForObject()
{ return detail::from(fearSelectBackendForObject()); }

/**
 * @brief Query availability of compiler backends.
 */
inline bool hasLLVM()
{ return fearHasBackend(FearBackendLlvm); }
inline bool hasCranelift()
{ return fearHasBackend(FearBackendCranelift); }

/**
 * @brief Builder class for constructing a function's body, basic blocks,
 * and instructions. Uses RAII to manage the lifecycle of the underlying
 * Fear Function definition.
 */
struct FunctionDef
{
    FunctionDef()
        : raw_(fearDefinitionCreate()),
          currentBlock_(fearGetEntryBlock(raw_))
    {
        if (!raw_)
            throw std::runtime_error("Failed to create Fear FunctionDef");
    }

    ~FunctionDef()
    {
        if (raw_) fearDefinitionDispose(raw_);
    }

    // Deleted copy semantics to maintain unique ownership
    FunctionDef(const FunctionDef&)            = delete;
    FunctionDef& operator=(const FunctionDef&) = delete;

    // Move semantics
    FunctionDef(FunctionDef&& other) noexcept
        : raw_(other.raw_), currentBlock_(other.currentBlock_)
    { other.raw_ = nullptr; }

    FunctionDef& operator=(FunctionDef&& other) noexcept
    {
        if (this != &other)
        {
            if (raw_) fearDefinitionDispose(raw_);
            raw_          = other.raw_;
            currentBlock_ = other.currentBlock_;
            other.raw_    = nullptr;
        }
        return *this;
    }

    RawFuncDef* getRaw() const { return raw_; }
    BlockId     getCurrentBlock() const { return currentBlock_; }

    // Block management
    BlockId     entryBlock() { return fearGetEntryBlock(getRaw()); }
    BlockId     createBlock() { return fearCreateBlock(getRaw()); }
    void        switchTo(BlockId id) { currentBlock_ = id; }

    // Parameter generation
    BlockId     funcParam(Type ty)
    { return fearCreateFuncParam(getRaw(), detail::into(ty)); }
    BlockId blockParam(Type ty)
    {
        return fearCreateBlockParam(getRaw(), getCurrentBlock(),
                                    detail::into(ty));
    }

    // Constants
    ValueId iconst(Type ty, int64_t val)
    {
        return fearCreateIntConst(getRaw(), getCurrentBlock(),
                                  detail::into(ty), val);
    }

    // Memory operations
    ValueId stack_alloca(Type ty)
    {
        return fearCreateAlloca(getRaw(), getCurrentBlock(),
                                detail::into(ty));
    }
    ValueId load(Type ty, ValueId ptr)
    {
        return fearCreateLoad(getRaw(), getCurrentBlock(),
                              detail::into(ty), ptr);
    }
    void store(ValueId ptr, ValueId value)
    { fearCreateStore(getRaw(), getCurrentBlock(), ptr, value); }

    ValueId vload(Type ty, ValueId ptr)
    {
        return fearCreateVolatileLoad(getRaw(), getCurrentBlock(),
                                      detail::into(ty), ptr);
    }
    void vstore(ValueId ptr, ValueId value)
    { fearCreateVolatileStore(getRaw(), getCurrentBlock(), ptr, value); }

    // Integer Arithmetic
    ValueId add(Type ty, ValueId a, ValueId b)
    {
        return fearCreateAdd(getRaw(), getCurrentBlock(), detail::into(ty),
                             a, b);
    }
    ValueId sub(Type ty, ValueId a, ValueId b)
    {
        return fearCreateSub(getRaw(), getCurrentBlock(), detail::into(ty),
                             a, b);
    }
    ValueId mul(Type ty, ValueId a, ValueId b)
    {
        return fearCreateMul(getRaw(), getCurrentBlock(), detail::into(ty),
                             a, b);
    }
    ValueId div(Type ty, ValueId a, ValueId b)
    {
        return fearCreateDiv(getRaw(), getCurrentBlock(), detail::into(ty),
                             a, b);
    }
    ValueId udiv(Type ty, ValueId a, ValueId b)
    {
        return fearCreateUnsignedDiv(getRaw(), getCurrentBlock(),
                                     detail::into(ty), a, b);
    }
    ValueId rem(Type ty, ValueId a, ValueId b)
    {
        return fearCreateRem(getRaw(), getCurrentBlock(), detail::into(ty),
                             a, b);
    }
    ValueId urem(Type ty, ValueId a, ValueId b)
    {
        return fearCreateUnsignedRem(getRaw(), getCurrentBlock(),
                                     detail::into(ty), a, b);
    }

    // Floating-Point Arithmetic
    ValueId fadd(Type ty, ValueId a, ValueId b)
    {
        return fearCreateFloatAdd(getRaw(), getCurrentBlock(),
                                  detail::into(ty), a, b);
    }
    ValueId fsub(Type ty, ValueId a, ValueId b)
    {
        return fearCreateFloatSub(getRaw(), getCurrentBlock(),
                                  detail::into(ty), a, b);
    }
    ValueId fmul(Type ty, ValueId a, ValueId b)
    {
        return fearCreateFloatMul(getRaw(), getCurrentBlock(),
                                  detail::into(ty), a, b);
    }
    ValueId fdiv(Type ty, ValueId a, ValueId b)
    {
        return fearCreateFloatDiv(getRaw(), getCurrentBlock(),
                                  detail::into(ty), a, b);
    }
    ValueId frem(Type ty, ValueId a, ValueId b)
    {
        return fearCreateFloatRem(getRaw(), getCurrentBlock(),
                                  detail::into(ty), a, b);
    }

    // Bitwise and Shifts
    ValueId bnot(Type ty, ValueId v)
    {
        return fearCreateBitwiseNot(getRaw(), getCurrentBlock(),
                                    detail::into(ty), v);
    }
    ValueId band(Type ty, ValueId a, ValueId b)
    {
        return fearCreateBitwiseAnd(getRaw(), getCurrentBlock(),
                                    detail::into(ty), a, b);
    }
    ValueId bor(Type ty, ValueId a, ValueId b)
    {
        return fearCreateBitwiseOr(getRaw(), getCurrentBlock(),
                                   detail::into(ty), a, b);
    }
    ValueId bxor(Type ty, ValueId a, ValueId b)
    {
        return fearCreateBitwiseXor(getRaw(), getCurrentBlock(),
                                    detail::into(ty), a, b);
    }
    ValueId shl(Type ty, ValueId a, ValueId b)
    {
        return fearCreateLogicalShiftLeft(getRaw(), getCurrentBlock(),
                                          detail::into(ty), a, b);
    }
    ValueId shr(Type ty, ValueId a, ValueId b)
    {
        return fearCreateLogicalShiftRight(getRaw(), getCurrentBlock(),
                                           detail::into(ty), a, b);
    }
    ValueId ashr(Type ty, ValueId a, ValueId b)
    {
        return fearCreateArithShiftRight(getRaw(), getCurrentBlock(),
                                         detail::into(ty), a, b);
    }

    // Control Flow / Termigators
    void jmp(BlockId target, const std::vector<ValueId>& params = {})
    {
        fearCreateJump(getRaw(), getCurrentBlock(), target, params.data(),
                       static_cast<uint32_t>(params.size()));
    }

    void jmpif(ValueId cond, BlockId true_block,
               const std::vector<ValueId>& true_args, BlockId false_block,
               const std::vector<ValueId>& false_args)
    {
        fearCreateCondJump(getRaw(), getCurrentBlock(), cond, true_block,
                           true_args.data(),
                           static_cast<uint32_t>(true_args.size()),
                           false_block, false_args.data(),
                           static_cast<uint32_t>(false_args.size()));
    }

    void ret(ValueId v) { fearCreateRet(getRaw(), getCurrentBlock(), v); }
    void ret() { fearCreateRetVoid(getRaw(), getCurrentBlock()); }

   private:
    RawFuncDef* raw_;
    BlockId     currentBlock_;
};

/**
 * @brief Represents a single compilation module containing function
 * declarations and definitions. Manages the lifecycle of the raw Fear
 * Module pointer via RAII.
 */
struct Module
{
    /**
     * @brief Creates an empty module with the specified name.
     */
    Module(std::string_view name)
        : raw_(fearModuleCreate(std::string(name).c_str()))
    {
        if (!raw_)
            throw std::runtime_error("Failed to create Fear Module");
    }

    /**
     * @brief Deserializes a module from a Fear binary file descriptor.
     */
    explicit Module(int fd) : raw_(fearReadBinaryFromFile(fd))
    {
        if (!raw_)
            throw std::runtime_error(
                "Failed to read Fear Module from file descriptor");
    }

    ~Module()
    {
        if (raw_) fearModuleDispose(raw_);
    }

    // Deleted copy semantics
    Module(const Module&)            = delete;
    Module& operator=(const Module&) = delete;

    // Move semantics
    Module(Module&& other) noexcept : raw_(other.raw_)
    { other.raw_ = nullptr; }

    Module& operator=(Module&& other) noexcept
    {
        if (this != &other)
        {
            if (raw_) fearModuleDispose(raw_);
            raw_       = other.raw_;
            other.raw_ = nullptr;
        }
        return *this;
    }

    RawModule* getRaw() const { return raw_; }

    /**
     * @brief Declares a function signature inside the module.
     */
    FuncId declareFunction(std::string_view         name,
                           const std::vector<Type>& params, Type returns,
                           Linkage linkage = Linkage::External)
    {
        std::vector<FearType> raw_params;
        raw_params.reserve(params.size());
        for (auto ty : params) { raw_params.push_back(detail::into(ty)); }

        std::string nt_name(name);

        return fearDeclareFunction(
            getRaw(), nt_name.c_str(), raw_params.data(),
            static_cast<uint32_t>(raw_params.size()),
            detail::into(returns), detail::into(linkage));
    }

    /**
     * @brief Attaches a compiled function definition body to a previously
     * declared function ID.
     */
    void defineFunction(FuncId id, const FunctionDef& def)
    { fearDefineFunction(getRaw(), id, def.getRaw()); }

    /**
     * @brief Optimizes the module using the specified optimization level.
     */
    unsigned optimize(OptLevel lvl)
    { return fearModuleOptimize(getRaw(), detail::into(lvl)); }

    // Serialization and Diagnostics
    void        dumpToFile(int fd) { fearDumpToFile(getRaw(), fd); }

    /**
     * @brief Dumps a plain-text IR representation of the module to an
     * std::string.
     */
    std::string dumpToString()
    {
        char* raw = fearDumpToString(getRaw());
        if (!raw) return "";
        std::string out{raw};
        fearStringDispose(raw);
        return out;
    }

    void binaryDumpToFile(int fd) { fearBinaryDumpToFile(getRaw(), fd); }

    /**
     * @brief Serializes the module into the compiler's native binary
     * format as a vector of bytes.
     */
    std::vector<uint8_t> binaryDumpToBuffer()
    {
        size_t   size = 0;
        uint8_t* raw  = fearBinaryDumpToBuffer(getRaw(), &size);
        if (!raw) return {};
        std::vector<uint8_t> out{raw, raw + size};
        fearBufferDispose(raw, size);
        return out;
    }

    /**
     * @brief Compiles and emits a machine code object file into the
     * specified file descriptor.
     */
    int emitObject(OptLevel opt, int fd,
                   Backend backend = selectBackendForObject())
    {
        return fearEmitObject(getRaw(), detail::into(backend),
                              detail::into(opt), fd);
    }

   private:
    RawModule* raw_;
};

/**
 * @brief High-level abstraction for handling functions inside a Module.
 */
struct Function
{
    Function(Module* parent, FuncId id) : parent_(parent), id_(id) {}

    /**
     * @brief Helper to declare a new function and return its high-level
     * wrapper object.
     */
    static Function declare(Module* m, std::string_view name,
                            const std::vector<Type>& params, Type returns,
                            Linkage linkage = Linkage::External)
    {
        FuncId id = m->declareFunction(name, params, returns, linkage);
        return Function(m, id);
    }

    /**
     * @brief Defines the function body using the provided FunctionDef
     * builder.
     */
    void define(const FunctionDef& def)
    { parent_->defineFunction(getId(), def); }

    /**
     * @brief Configures the target calling convention for the function.
     */
    void setCallingConvention(CallConv cc)
    { fearFunctionSetCC(parent_->getRaw(), getId(), detail::into(cc)); }

    const Module* getParent() { return parent_; }
    FuncId        getId() const { return id_; }

   private:
    Module* parent_;
    FuncId  id_;
};

}  // namespace fear