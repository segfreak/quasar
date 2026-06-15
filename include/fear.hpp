#pragma once

#include <cstdint>
#include <optional>
#include <stdexcept>
#include <string>
#include <string_view>
#include <vector>

#include "fear.h"

namespace fear
{

/**
 * @brief Initializes logging.
 */
inline void initLogging()
{ fearInitLogging(); }

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

/**
 * @brief Integer comparison predicates for the `icmp` instruction.
 */
enum class IntPredicate
{
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    ULt,
    ULe,
    UGt,
    UGe,
};

/**
 * @brief Float comparison predicates for the `fcmp` instruction.
 */
enum class FloatPredicate
{
    Ord,
    OEq,
    ONe,
    OLt,
    OLe,
    OGt,
    OGe,
    Uno,
    UEq,
    UNe,
    ULt,
    ULe,
    UGt,
    UGe,
};

// Type aliases for compiler primitives
struct ValueId
{
    ValueId(FearValueId value) : value_(value) {}

    FearValueId getRaw() const { return value_; }

   private:
    FearValueId value_;
};

struct BlockId
{
    BlockId(FearBlockId value) : value_(value) {}

    FearBlockId getRaw() const { return value_; }

   private:
    FearBlockId value_;
};

struct FuncId
{
    FuncId(FearFuncId value) : value_(value) {}

    FearFuncId getRaw() const { return value_; }

   private:
    FearFuncId value_;
};

using RawFuncDef = FearFunctionDef;
using RawModule  = FearModule;

namespace detail
{

/**
 * @brief Converts an optional string_view into a C-style string pointer.
 *
 * This helper is intended for FFI boundaries where a `const char*`
 * is required.
 *
 * @param opt_view Optional string_view input.
 *
 * @return Pointer to the underlying character data if the value is
 * present, otherwise `nullptr`.
 *
 @warning The returned pointer is not guaranteed to be null-terminated.
 *          It is only safe to use as a C-string if the original src
 *          is known to be null-terminated (e.g. std::string::c_str(),
 *          string literals, or similar sources).
 *
 * @warning The returned pointer does not own the data and becomes invalid
 *          if the underlying string_view storage is destroyed or modified.
 */
inline const char* unwrap_cstr(std::optional<std::string_view> opt_view)
{ return opt_view ? opt_view->data() : nullptr; }

/**
 * @brief Converts C++ high-level enums to raw C API constants.
 */
inline FearIntCmp into(IntPredicate pred)
{
    switch (pred)
    {
        case IntPredicate::Eq:
            return FearIntCmpEq;
        case IntPredicate::Ne:
            return FearIntCmpNe;
        case IntPredicate::Lt:
            return FearIntCmpLt;
        case IntPredicate::Le:
            return FearIntCmpLe;
        case IntPredicate::Gt:
            return FearIntCmpGt;
        case IntPredicate::Ge:
            return FearIntCmpGe;
        case IntPredicate::ULt:
            return FearIntCmpULt;
        case IntPredicate::ULe:
            return FearIntCmpULe;
        case IntPredicate::UGt:
            return FearIntCmpUGt;
        case IntPredicate::UGe:
            return FearIntCmpUGe;
    }
    throw std::runtime_error("Unknown IntPredicate");
}

inline FearFloatCmp into(FloatPredicate pred)
{
    switch (pred)
    {
        case FloatPredicate::Ord:
            return FearFloatCmpOrd;
        case FloatPredicate::OEq:
            return FearFloatCmpOrdEq;
        case FloatPredicate::ONe:
            return FearFloatCmpOrdNe;
        case FloatPredicate::OLt:
            return FearFloatCmpOrdLt;
        case FloatPredicate::OLe:
            return FearFloatCmpOrdLe;
        case FloatPredicate::OGt:
            return FearFloatCmpOrdGt;
        case FloatPredicate::OGe:
            return FearFloatCmpOrdGe;
        case FloatPredicate::Uno:
            return FearFloatCmpUno;
        case FloatPredicate::UEq:
            return FearFloatCmpUnoEq;
        case FloatPredicate::UNe:
            return FearFloatCmpUnoNe;
        case FloatPredicate::ULt:
            return FearFloatCmpUnoLt;
        case FloatPredicate::ULe:
            return FearFloatCmpUnoLe;
        case FloatPredicate::UGt:
            return FearFloatCmpUnoGt;
        case FloatPredicate::UGe:
            return FearFloatCmpUnoGe;
    }
    throw std::runtime_error("Unknown FloatPredicate");
}

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

static std::vector<FearValueId> into(const std::vector<ValueId>& values)
{
    std::vector<FearValueId> result;
    result.reserve(values.size());

    for (auto v : values) result.push_back(v.getRaw());

    return result;
}

/**
 * @brief Converts raw C API constants back to C++ high-level enums.
 */

inline IntPredicate from(FearIntCmp pred)
{
    switch (pred)
    {
        case FearIntCmpEq:
            return IntPredicate::Eq;
        case FearIntCmpNe:
            return IntPredicate::Ne;
        case FearIntCmpLt:
            return IntPredicate::Lt;
        case FearIntCmpLe:
            return IntPredicate::Le;
        case FearIntCmpGt:
            return IntPredicate::Gt;
        case FearIntCmpGe:
            return IntPredicate::Ge;
        case FearIntCmpULt:
            return IntPredicate::ULt;
        case FearIntCmpULe:
            return IntPredicate::ULe;
        case FearIntCmpUGt:
            return IntPredicate::UGt;
        case FearIntCmpUGe:
            return IntPredicate::UGe;
    }
    throw std::runtime_error("Unknown FearIntCmp");
}

inline FloatPredicate from(FearFloatCmp pred)
{
    switch (pred)
    {
        case FearFloatCmpOrd:
            return FloatPredicate::Ord;
        case FearFloatCmpOrdEq:
            return FloatPredicate::OEq;
        case FearFloatCmpOrdNe:
            return FloatPredicate::ONe;
        case FearFloatCmpOrdLt:
            return FloatPredicate::OLt;
        case FearFloatCmpOrdLe:
            return FloatPredicate::OLe;
        case FearFloatCmpOrdGt:
            return FloatPredicate::OGt;
        case FearFloatCmpOrdGe:
            return FloatPredicate::OGe;
        case FearFloatCmpUno:
            return FloatPredicate::Uno;
        case FearFloatCmpUnoEq:
            return FloatPredicate::UEq;
        case FearFloatCmpUnoNe:
            return FloatPredicate::UNe;
        case FearFloatCmpUnoLt:
            return FloatPredicate::ULt;
        case FearFloatCmpUnoLe:
            return FloatPredicate::ULe;
        case FearFloatCmpUnoGt:
            return FloatPredicate::UGt;
        case FearFloatCmpUnoGe:
            return FloatPredicate::UGe;
    }
    throw std::runtime_error("Unknown FearFloatCmp");
}

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
    ValueId     funcParam(Type ty)
    { return fearCreateFuncParam(getRaw(), detail::into(ty)); }
    ValueId blockParam(Type ty)
    {
        return fearCreateBlockParam(getRaw(), getCurrentBlock().getRaw(),
                                    detail::into(ty));
    }

    // Constants
    ValueId iconst(Type ty, int64_t val)
    {
        return fearCreateIntConst(getRaw(), getCurrentBlock().getRaw(),
                                  detail::into(ty), val);
    }
    ValueId fconst(Type ty, double val)
    {
        return fearCreateFloatConst(getRaw(), getCurrentBlock().getRaw(),
                                    detail::into(ty), val);
    }

    // Memory operations
    ValueId alloca(Type ty)
    {
        return fearCreateAlloca(getRaw(), getCurrentBlock().getRaw(),
                                detail::into(ty));
    }
    ValueId load(Type ty, ValueId ptr)
    {
        return fearCreateLoad(getRaw(), getCurrentBlock().getRaw(),
                              detail::into(ty), ptr.getRaw());
    }
    void store(ValueId ptr, ValueId value)
    {
        fearCreateStore(getRaw(), getCurrentBlock().getRaw(), ptr.getRaw(),
                        value.getRaw());
    }

    ValueId vload(Type ty, ValueId ptr)
    {
        return fearCreateVolatileLoad(getRaw(), getCurrentBlock().getRaw(),
                                      detail::into(ty), ptr.getRaw());
    }
    void vstore(ValueId ptr, ValueId value)
    {
        fearCreateVolatileStore(getRaw(), getCurrentBlock().getRaw(),
                                ptr.getRaw(), value.getRaw());
    }

    // Integer Arithmetic
    ValueId add(Type ty, ValueId a, ValueId b)
    {
        return fearCreateAdd(getRaw(), getCurrentBlock().getRaw(),
                             detail::into(ty), a.getRaw(), b.getRaw());
    }
    ValueId sub(Type ty, ValueId a, ValueId b)
    {
        return fearCreateSub(getRaw(), getCurrentBlock().getRaw(),
                             detail::into(ty), a.getRaw(), b.getRaw());
    }
    ValueId mul(Type ty, ValueId a, ValueId b)
    {
        return fearCreateMul(getRaw(), getCurrentBlock().getRaw(),
                             detail::into(ty), a.getRaw(), b.getRaw());
    }
    ValueId div(Type ty, ValueId a, ValueId b)
    {
        return fearCreateDiv(getRaw(), getCurrentBlock().getRaw(),
                             detail::into(ty), a.getRaw(), b.getRaw());
    }
    ValueId udiv(Type ty, ValueId a, ValueId b)
    {
        return fearCreateUnsignedDiv(getRaw(), getCurrentBlock().getRaw(),
                                     detail::into(ty), a.getRaw(),
                                     b.getRaw());
    }
    ValueId rem(Type ty, ValueId a, ValueId b)
    {
        return fearCreateRem(getRaw(), getCurrentBlock().getRaw(),
                             detail::into(ty), a.getRaw(), b.getRaw());
    }
    ValueId urem(Type ty, ValueId a, ValueId b)
    {
        return fearCreateUnsignedRem(getRaw(), getCurrentBlock().getRaw(),
                                     detail::into(ty), a.getRaw(),
                                     b.getRaw());
    }

    // Floating-Point Arithmetic
    ValueId fadd(Type ty, ValueId a, ValueId b)
    {
        return fearCreateFloatAdd(getRaw(), getCurrentBlock().getRaw(),
                                  detail::into(ty), a.getRaw(),
                                  b.getRaw());
    }
    ValueId fsub(Type ty, ValueId a, ValueId b)
    {
        return fearCreateFloatSub(getRaw(), getCurrentBlock().getRaw(),
                                  detail::into(ty), a.getRaw(),
                                  b.getRaw());
    }
    ValueId fmul(Type ty, ValueId a, ValueId b)
    {
        return fearCreateFloatMul(getRaw(), getCurrentBlock().getRaw(),
                                  detail::into(ty), a.getRaw(),
                                  b.getRaw());
    }
    ValueId fdiv(Type ty, ValueId a, ValueId b)
    {
        return fearCreateFloatDiv(getRaw(), getCurrentBlock().getRaw(),
                                  detail::into(ty), a.getRaw(),
                                  b.getRaw());
    }
    ValueId frem(Type ty, ValueId a, ValueId b)
    {
        return fearCreateFloatRem(getRaw(), getCurrentBlock().getRaw(),
                                  detail::into(ty), a.getRaw(),
                                  b.getRaw());
    }

    // Bitwise and Shifts
    ValueId bnot(Type ty, ValueId v)
    {
        return fearCreateBitNot(getRaw(), getCurrentBlock().getRaw(),
                                detail::into(ty), v.getRaw());
    }
    ValueId band(Type ty, ValueId a, ValueId b)
    {
        return fearCreateBitAnd(getRaw(), getCurrentBlock().getRaw(),
                                detail::into(ty), a.getRaw(), b.getRaw());
    }
    ValueId bor(Type ty, ValueId a, ValueId b)
    {
        return fearCreateBitOr(getRaw(), getCurrentBlock().getRaw(),
                               detail::into(ty), a.getRaw(), b.getRaw());
    }
    ValueId bxor(Type ty, ValueId a, ValueId b)
    {
        return fearCreateBitXor(getRaw(), getCurrentBlock().getRaw(),
                                detail::into(ty), a.getRaw(), b.getRaw());
    }
    ValueId shl(Type ty, ValueId a, ValueId b)
    {
        return fearCreateShl(getRaw(), getCurrentBlock().getRaw(),
                             detail::into(ty), a.getRaw(), b.getRaw());
    }
    ValueId shr(Type ty, ValueId a, ValueId b)
    {
        return fearCreateShr(getRaw(), getCurrentBlock().getRaw(),
                             detail::into(ty), a.getRaw(), b.getRaw());
    }
    ValueId ashr(Type ty, ValueId a, ValueId b)
    {
        return fearCreateArithShr(getRaw(), getCurrentBlock().getRaw(),
                                  detail::into(ty), a.getRaw(),
                                  b.getRaw());
    }

    ValueId icmp(IntPredicate pred, ValueId left, ValueId right)
    {
        return fearCreateIntCompare(getRaw(), getCurrentBlock().getRaw(),
                                    detail::into(pred), left.getRaw(),
                                    right.getRaw());
    }

    ValueId fcmp(FloatPredicate pred, ValueId left, ValueId right)
    {
        return fearCreateFloatCompare(getRaw(), getCurrentBlock().getRaw(),
                                      detail::into(pred), left.getRaw(),
                                      right.getRaw());
    }

    // Control Flow / Terminators
    void jmp(BlockId target, const std::vector<ValueId>& params = {})
    {
        auto raw_params = detail::into(params);
        fearCreateJump(getRaw(), getCurrentBlock().getRaw(),
                       target.getRaw(), raw_params.data(),
                       static_cast<uint32_t>(raw_params.size()));
    }

    void jmpif(ValueId cond, BlockId true_block,
               const std::vector<ValueId>& true_args, BlockId false_block,
               const std::vector<ValueId>& false_args)
    {
        auto raw_true_args  = detail::into(true_args);
        auto raw_false_args = detail::into(/* bug-fix */ false_args);

        fearCreateCondJump(getRaw(), getCurrentBlock().getRaw(),
                           cond.getRaw(), true_block.getRaw(),
                           raw_true_args.data(),
                           static_cast<uint32_t>(raw_true_args.size()),
                           false_block.getRaw(), raw_false_args.data(),
                           static_cast<uint32_t>(raw_false_args.size()));
    }

    void ret(ValueId v)
    { fearCreateRet(getRaw(), getCurrentBlock().getRaw(), v.getRaw()); }
    void ret() { fearCreateRetVoid(getRaw(), getCurrentBlock().getRaw()); }

    std::optional<ValueId> call(FuncId func, Type ret,
                                std::vector<ValueId> params)
    {
        auto raw_params = detail::into(params);
        auto tmp        = fearCreateCall(
            getRaw(), getCurrentBlock().getRaw(), func.getRaw(),
            detail::into(ret), raw_params.data(),
            static_cast<uint32_t>(raw_params.size()));
        if (ret == Type::Void) { return std::nullopt; }
        return tmp;
    }

    ValueId undef(Type ty)
    {
        return fearCreateUndef(getRaw(), getCurrentBlock().getRaw(),
                               detail::into(ty));
    }

    ValueId select(Type ty, ValueId cond, ValueId then_value,
                   ValueId else_value)
    {
        return fearCreateSelect(getRaw(), getCurrentBlock().getRaw(),
                                detail::into(ty), cond.getRaw(),
                                then_value.getRaw(), else_value.getRaw());
    }

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
    { fearDefineFunction(getRaw(), id.getRaw(), def.getRaw()); }

    /**
     * @brief Optimizes the module using the specified optimization level.
     */
    unsigned optimize(OptLevel lvl)
    { return fearModuleOptimize(getRaw(), detail::into(lvl)); }

    /**
     * @brief Verifys the module for correctness and consistency.
     * @returns Returns the number of errors found.
     */
    unsigned    verify() { return fearModuleVerify(getRaw()); }

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
     *
     * @param opt Optimization level.
     * @param fd Output file descriptor.
     * @param is_pic Generate position-independent code.
     * @param triple Target triple (e.g. "x86_64-unknown-linux-gnu").
     *               If std::nullopt, the host target triple is used.
     * @param cpu Target CPU name (e.g. "tigerlake", "znver4").
     *            If std::nullopt, the backend default generic CPU is used.
     * @param backend Target backend. Defaults to the backend selected
     *                for the current host.
     *
     * @return 0 on success, non-zero on failure.
     */
    int emitObject(OptLevel opt, int fd, bool is_pic,
                   std::optional<std::string_view> triple,
                   std::optional<std::string_view> cpu,
                   Backend backend = selectBackendForObject())
    {
        return fearEmitObject(
            getRaw(), detail::into(backend), detail::into(opt), is_pic,
            detail::unwrap_cstr(triple), detail::unwrap_cstr(cpu), fd);
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
     * @brief Helper to declare a new function and return its
     * high-level wrapper object.
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
     * @brief Configures the target calling convention for the
     * function.
     */
    void setCallingConvention(CallConv cc)
    {
        fearFunctionSetCC(parent_->getRaw(), getId().getRaw(),
                          detail::into(cc));
    }

    const Module* getParent() { return parent_; }
    FuncId        getId() const { return id_; }

   private:
    Module* parent_;
    FuncId  id_;
};

}  // namespace fear