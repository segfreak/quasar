#pragma once

#include "fearc.h"
#include <stdexcept>
#include <string>
#include <string_view>
#include <vector>

namespace fear
{

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

enum class Linkage
{
  External,
  Internal,
  Weak,
};

enum class OptLevel
{
  None,
  Default,
  Full,
};

enum class CallConv
{
  C,
  SysV,
  MsAbi,
};

inline FearType
__unwrap (Type ty)
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
  throw std::runtime_error ("Unknown Fear Type");
}

inline FearLinkage
__unwrap (Linkage linkage)
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
  throw std::runtime_error ("Unknown Fear Linkage");
}

inline FearOptLevel
__unwrap (OptLevel level)
{
  switch (level)
  {
  case OptLevel::None:
    return FearOptLevelNone;
  case OptLevel::Default:
    return FearOptLevelDefault;
  case OptLevel::Full:
    return FearOptLevelFull;
  }
  throw std::runtime_error ("Unknown Fear OptLevel");
}

inline FearCallConv
__unwrap (CallConv cc)
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
  throw std::runtime_error ("Unknown Fear CallConv");
}

using ValueId    = FearValueId;
using BlockId    = FearBlockId;
using FuncId     = FearFuncId;

using RawFuncDef = FearFunctionDef;
using RawModule  = FearModule;

inline bool
hasLLVM ()
{ return fearHasBackend (FearBackendLlvm); }
inline bool
hasCranelift ()
{ return fearHasBackend (FearBackendCranelift); }

struct FunctionDef
{
  FunctionDef () : raw_ (fearDefinitionCreate ())
  {
    if (!raw_)
      throw std::runtime_error ("Failed to create Fear FunctionDef");
  }

  ~FunctionDef ()
  {
    if (raw_)
      fearDefinitionDispose (raw_);
  }

  FunctionDef (const FunctionDef &)            = delete;
  FunctionDef &operator= (const FunctionDef &) = delete;

  FunctionDef (FunctionDef &&other) noexcept : raw_ (other.raw_)
  { other.raw_ = nullptr; }
  FunctionDef &
  operator= (FunctionDef &&other) noexcept
  {
    if (this != &other)
    {
      if (raw_)
        fearDefinitionDispose (raw_);
      raw_       = other.raw_;
      other.raw_ = nullptr;
    }
    return *this;
  }

  RawFuncDef *
  raw () const
  { return raw_; }

  BlockId
  getEntryBlock ()
  { return fearGetEntryBlock (raw_); }
  BlockId
  createBlock ()
  { return fearCreateBlock (raw_); }
  BlockId
  createFuncParam (Type ty)
  { return fearCreateFuncParam (raw_, __unwrap (ty)); }
  BlockId
  createBlockParam (BlockId block, Type ty)
  { return fearCreateBlockParam (raw_, block, __unwrap (ty)); }

  ValueId
  createIntConst (BlockId parent, Type ty, int64_t val)
  { return fearCreateIntConst (raw_, parent, __unwrap (ty), val); }

  ValueId
  createAlloca (BlockId parent, Type ty)
  { return fearCreateAlloca (raw_, parent, __unwrap (ty)); }
  ValueId
  createLoad (BlockId parent, Type ty, ValueId ptr)
  { return fearCreateLoad (raw_, parent, __unwrap (ty), ptr); }
  void
  createStore (BlockId parent, ValueId ptr, ValueId value)
  { fearCreateStore (raw_, parent, ptr, value); }
  ValueId
  createVolatileLoad (BlockId parent, Type ty, ValueId ptr)
  { return fearCreateVolatileLoad (raw_, parent, __unwrap (ty), ptr); }
  void
  createVolatileStore (BlockId parent, ValueId ptr, ValueId value)
  { fearCreateVolatileStore (raw_, parent, ptr, value); }

  ValueId
  createAdd (BlockId parent, Type ty, ValueId a, ValueId b)
  { return fearCreateAdd (raw_, parent, __unwrap (ty), a, b); }
  ValueId
  createSub (BlockId parent, Type ty, ValueId a, ValueId b)
  { return fearCreateSub (raw_, parent, __unwrap (ty), a, b); }
  ValueId
  createMul (BlockId parent, Type ty, ValueId a, ValueId b)
  { return fearCreateMul (raw_, parent, __unwrap (ty), a, b); }
  ValueId
  createDiv (BlockId parent, Type ty, ValueId a, ValueId b)
  { return fearCreateDiv (raw_, parent, __unwrap (ty), a, b); }
  ValueId
  createUnsignedDiv (BlockId parent, Type ty, ValueId a, ValueId b)
  { return fearCreateUnsignedDiv (raw_, parent, __unwrap (ty), a, b); }
  ValueId
  createRem (BlockId parent, Type ty, ValueId a, ValueId b)
  { return fearCreateRem (raw_, parent, __unwrap (ty), a, b); }
  ValueId
  createUnsignedRem (BlockId parent, Type ty, ValueId a, ValueId b)
  { return fearCreateUnsignedRem (raw_, parent, __unwrap (ty), a, b); }

  ValueId
  createFloatAdd (BlockId parent, Type ty, ValueId a, ValueId b)
  { return fearCreateFloatAdd (raw_, parent, __unwrap (ty), a, b); }
  ValueId
  createFloatSub (BlockId parent, Type ty, ValueId a, ValueId b)
  { return fearCreateFloatSub (raw_, parent, __unwrap (ty), a, b); }
  ValueId
  createFloatMul (BlockId parent, Type ty, ValueId a, ValueId b)
  { return fearCreateFloatMul (raw_, parent, __unwrap (ty), a, b); }
  ValueId
  createFloatDiv (BlockId parent, Type ty, ValueId a, ValueId b)
  { return fearCreateFloatDiv (raw_, parent, __unwrap (ty), a, b); }
  ValueId
  createFloatRem (BlockId parent, Type ty, ValueId a, ValueId b)
  { return fearCreateFloatRem (raw_, parent, __unwrap (ty), a, b); }

  ValueId
  createBitwiseNot (BlockId parent, Type ty, ValueId v)
  { return fearCreateBitwiseNot (raw_, parent, __unwrap (ty), v); }
  ValueId
  createBitwiseAnd (BlockId parent, Type ty, ValueId a, ValueId b)
  { return fearCreateBitwiseAnd (raw_, parent, __unwrap (ty), a, b); }
  ValueId
  createBitwiseOr (BlockId parent, Type ty, ValueId a, ValueId b)
  { return fearCreateBitwiseOr (raw_, parent, __unwrap (ty), a, b); }
  ValueId
  createBitwiseXor (BlockId parent, Type ty, ValueId a, ValueId b)
  { return fearCreateBitwiseXor (raw_, parent, __unwrap (ty), a, b); }
  ValueId
  createLogicalShiftLeft (BlockId parent, Type ty, ValueId a, ValueId b)
  {
    return fearCreateLogicalShiftLeft (raw_, parent, __unwrap (ty), a, b);
  }
  ValueId
  createLogicalShiftRight (BlockId parent, Type ty, ValueId a, ValueId b)
  {
    return fearCreateLogicalShiftRight (raw_, parent, __unwrap (ty), a, b);
  }
  ValueId
  createArithShiftRight (BlockId parent, Type ty, ValueId a, ValueId b)
  { return fearCreateArithShiftRight (raw_, parent, __unwrap (ty), a, b); }

  void
  createJump (BlockId parent, BlockId target,
              const std::vector<ValueId> &params = {})
  {
    fearCreateJump (raw_, parent, target, params.data (),
                    static_cast<uint32_t> (params.size ()));
  }

  void
  createCondJump (BlockId parent, ValueId cond, BlockId true_block,
                  const std::vector<ValueId> &true_args,
                  BlockId                     false_block,
                  const std::vector<ValueId> &false_args)
  {
    fearCreateCondJump (raw_, parent, cond, true_block, true_args.data (),
                        static_cast<uint32_t> (true_args.size ()),
                        false_block, false_args.data (),
                        static_cast<uint32_t> (false_args.size ()));
  }

  void
  createRet (BlockId parent, ValueId v)
  { fearCreateRet (raw_, parent, v); }
  void
  createRet (BlockId parent)
  { fearCreateRetVoid (raw_, parent); }

private:
  RawFuncDef *raw_;
};

struct Module
{
  Module (std::string_view name)
      : raw_ (fearModuleCreate (std::string (name).c_str ()))
  {
    if (!raw_)
      throw std::runtime_error ("Failed to create Fear Module");
  }

  explicit Module (int fd) : raw_ (fearReadBinaryFromFile (fd))
  {
    if (!raw_)
      throw std::runtime_error (
          "Failed to read Fear Module from file descriptor");
  }

  ~Module ()
  {
    if (raw_)
      fearModuleDispose (raw_);
  }

  Module (const Module &)            = delete;
  Module &operator= (const Module &) = delete;

  Module (Module &&other) noexcept : raw_ (other.raw_)
  { other.raw_ = nullptr; }
  Module &
  operator= (Module &&other) noexcept
  {
    if (this != &other)
    {
      if (raw_)
        fearModuleDispose (raw_);
      raw_       = other.raw_;
      other.raw_ = nullptr;
    }
    return *this;
  }

  RawModule *
  raw () const
  { return raw_; }

  FuncId
  declareFunction (std::string_view name, const std::vector<Type> &params,
                   Type returns, Linkage linkage = Linkage::External)
  {
    std::vector<FearType> raw_params;
    raw_params.reserve (params.size ());
    for (auto ty : params)
    {
      raw_params.push_back (__unwrap (ty));
    }

    std::string nt_name (name);

    return fearDeclareFunction (raw_, nt_name.c_str (), raw_params.data (),
                                static_cast<uint32_t> (raw_params.size ()),
                                __unwrap (returns), __unwrap (linkage));
  }

  void
  functionSetCC (FuncId id, CallConv cc)
  { fearFunctionSetCC (raw_, id, __unwrap (cc)); }

  void
  defineFunction (FuncId id, const FunctionDef &def)
  { fearDefineFunction (raw_, id, def.raw ()); }

  unsigned
  optimize ()
  { return fearModuleOptimize (raw_); }

  void
  dumpToFile (int fd)
  { fearDumpToFile (raw_, fd); }

  void
  binaryDumpToFile (int fd)
  { fearBinaryDumpToFile (raw_, fd); }

  int
  emitObject (OptLevel opt, int fd)
  {
    return fearEmitObject (raw_, fearSelectBackend (), __unwrap (opt), fd);
  }

private:
  RawModule *raw_;
};

} // namespace fear