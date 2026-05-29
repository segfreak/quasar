#pragma once

#include "fear.h"

#include <stdexcept>
#include <string>
#include <string_view>
#include <vector>

namespace fear
{

// forward declarations
struct Module;
struct Function;
struct FunctionDef;

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
    return FearOptNone;
  case OptLevel::Default:
    return FearOptDefault;
  case OptLevel::Full:
    return FearOptFull;
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
  FunctionDef ()
      : raw_ (fearDefinitionCreate ()),
        currentBlock_ (fearGetEntryBlock (raw_))
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
  getRaw () const
  { return raw_; }
  BlockId
  getCurrentBlock () const
  { return currentBlock_; }

  BlockId
  entryBlock ()
  { return fearGetEntryBlock (getRaw ()); }
  BlockId
  createBlock ()
  { return fearCreateBlock (getRaw ()); }
  void
  switchTo (BlockId id)
  { currentBlock_ = id; }

  BlockId
  funcParam (Type ty)
  { return fearCreateFuncParam (getRaw (), __unwrap (ty)); }
  BlockId
  blockParam (Type ty)
  {
    return fearCreateBlockParam (getRaw (), getCurrentBlock (),
                                 __unwrap (ty));
  }

  ValueId
  iconst (Type ty, int64_t val)
  {
    return fearCreateIntConst (getRaw (), getCurrentBlock (),
                               __unwrap (ty), val);
  }

  ValueId
  stack_alloca (Type ty)
  {
    return fearCreateAlloca (getRaw (), getCurrentBlock (), __unwrap (ty));
  }
  ValueId
  load (Type ty, ValueId ptr)
  {
    return fearCreateLoad (getRaw (), getCurrentBlock (), __unwrap (ty),
                           ptr);
  }
  void
  store (ValueId ptr, ValueId value)
  { fearCreateStore (getRaw (), getCurrentBlock (), ptr, value); }
  ValueId
  vload (Type ty, ValueId ptr)
  {
    return fearCreateVolatileLoad (getRaw (), getCurrentBlock (),
                                   __unwrap (ty), ptr);
  }
  void
  vstore (ValueId ptr, ValueId value)
  { fearCreateVolatileStore (getRaw (), getCurrentBlock (), ptr, value); }

  ValueId
  add (Type ty, ValueId a, ValueId b)
  {
    return fearCreateAdd (getRaw (), getCurrentBlock (), __unwrap (ty), a,
                          b);
  }
  ValueId
  sub (Type ty, ValueId a, ValueId b)
  {
    return fearCreateSub (getRaw (), getCurrentBlock (), __unwrap (ty), a,
                          b);
  }
  ValueId
  mul (Type ty, ValueId a, ValueId b)
  {
    return fearCreateMul (getRaw (), getCurrentBlock (), __unwrap (ty), a,
                          b);
  }
  ValueId
  div (Type ty, ValueId a, ValueId b)
  {
    return fearCreateDiv (getRaw (), getCurrentBlock (), __unwrap (ty), a,
                          b);
  }
  ValueId
  udiv (Type ty, ValueId a, ValueId b)
  {
    return fearCreateUnsignedDiv (getRaw (), getCurrentBlock (),
                                  __unwrap (ty), a, b);
  }
  ValueId
  rem (Type ty, ValueId a, ValueId b)
  {
    return fearCreateRem (getRaw (), getCurrentBlock (), __unwrap (ty), a,
                          b);
  }
  ValueId
  urem (Type ty, ValueId a, ValueId b)
  {
    return fearCreateUnsignedRem (getRaw (), getCurrentBlock (),
                                  __unwrap (ty), a, b);
  }

  ValueId
  fadd (Type ty, ValueId a, ValueId b)
  {
    return fearCreateFloatAdd (getRaw (), getCurrentBlock (),
                               __unwrap (ty), a, b);
  }
  ValueId
  fsub (Type ty, ValueId a, ValueId b)
  {
    return fearCreateFloatSub (getRaw (), getCurrentBlock (),
                               __unwrap (ty), a, b);
  }
  ValueId
  fmul (Type ty, ValueId a, ValueId b)
  {
    return fearCreateFloatMul (getRaw (), getCurrentBlock (),
                               __unwrap (ty), a, b);
  }
  ValueId
  fdiv (Type ty, ValueId a, ValueId b)
  {
    return fearCreateFloatDiv (getRaw (), getCurrentBlock (),
                               __unwrap (ty), a, b);
  }
  ValueId
  frem (Type ty, ValueId a, ValueId b)
  {
    return fearCreateFloatRem (getRaw (), getCurrentBlock (),
                               __unwrap (ty), a, b);
  }

  ValueId
  bnot (Type ty, ValueId v)
  {
    return fearCreateBitwiseNot (getRaw (), getCurrentBlock (),
                                 __unwrap (ty), v);
  }
  ValueId
  band (Type ty, ValueId a, ValueId b)
  {
    return fearCreateBitwiseAnd (getRaw (), getCurrentBlock (),
                                 __unwrap (ty), a, b);
  }
  ValueId
  bor (Type ty, ValueId a, ValueId b)
  {
    return fearCreateBitwiseOr (getRaw (), getCurrentBlock (),
                                __unwrap (ty), a, b);
  }
  ValueId
  bxor (Type ty, ValueId a, ValueId b)
  {
    return fearCreateBitwiseXor (getRaw (), getCurrentBlock (),
                                 __unwrap (ty), a, b);
  }
  ValueId
  shl (Type ty, ValueId a, ValueId b)
  {
    return fearCreateLogicalShiftLeft (getRaw (), getCurrentBlock (),
                                       __unwrap (ty), a, b);
  }
  ValueId
  shr (Type ty, ValueId a, ValueId b)
  {
    return fearCreateLogicalShiftRight (getRaw (), getCurrentBlock (),
                                        __unwrap (ty), a, b);
  }
  ValueId
  ashr (Type ty, ValueId a, ValueId b)
  {
    return fearCreateArithShiftRight (getRaw (), getCurrentBlock (),
                                      __unwrap (ty), a, b);
  }

  void
  jmp (BlockId target, const std::vector<ValueId> &params = {})
  {
    fearCreateJump (getRaw (), getCurrentBlock (), target, params.data (),
                    static_cast<uint32_t> (params.size ()));
  }

  void
  jmpif (ValueId cond, BlockId true_block,
         const std::vector<ValueId> &true_args, BlockId false_block,
         const std::vector<ValueId> &false_args)
  {
    fearCreateCondJump (
        getRaw (), getCurrentBlock (), cond, true_block, true_args.data (),
        static_cast<uint32_t> (true_args.size ()), false_block,
        false_args.data (), static_cast<uint32_t> (false_args.size ()));
  }

  void
  ret (ValueId v)
  { fearCreateRet (getRaw (), getCurrentBlock (), v); }
  void
  ret ()
  { fearCreateRetVoid (getRaw (), getCurrentBlock ()); }

private:
  RawFuncDef *raw_;
  BlockId     currentBlock_;
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
  getRaw () const
  { return raw_; }

  unsigned
  optimize (OptLevel lvl)
  { return fearModuleOptimize (getRaw (), __unwrap (lvl)); }

  void
  dumpToFile (int fd)
  { fearDumpToFile (getRaw (), fd); }

  void
  binaryDumpToFile (int fd)
  { fearBinaryDumpToFile (getRaw (), fd); }

  int
  emitObject (OptLevel opt, int fd)
  {
    return fearEmitObject (getRaw (), fearSelectBackendForObject (),
                           __unwrap (opt), fd);
  }

private:
  friend class Function;

  FuncId
  _declareFunction (std::string_view name, const std::vector<Type> &params,
                    Type returns, Linkage linkage = Linkage::External)
  {
    std::vector<FearType> raw_params;
    raw_params.reserve (params.size ());
    for (auto ty : params)
    {
      raw_params.push_back (__unwrap (ty));
    }

    std::string nt_name (name);

    return fearDeclareFunction (getRaw (), nt_name.c_str (),
                                raw_params.data (),
                                static_cast<uint32_t> (raw_params.size ()),
                                __unwrap (returns), __unwrap (linkage));
  }

  void
  _defineFunction (FuncId id, const FunctionDef &def)
  { fearDefineFunction (getRaw (), id, def.getRaw ()); }

  RawModule *raw_;
};

struct Function
{
  Function (Module *parent, FuncId id) : parent_ (parent), id_ (id) {}

  static Function
  declare (Module *m, std::string_view name,
           const std::vector<Type> &params, Type returns,
           Linkage linkage = Linkage::External)
  {
    FuncId id = m->_declareFunction (name, params, returns, linkage);
    return Function (m, id);
  }

  void
  define (const FunctionDef &def)
  { parent_->_defineFunction (getId (), def); }

  void
  setCallingConvention (FuncId id, CallConv cc)
  { fearFunctionSetCC (parent_->getRaw (), id, __unwrap (cc)); }

  const Module *
  getParent ()
  { return parent_; }

  FuncId
  getId () const
  { return id_; }

private:
  Module *parent_;
  FuncId  id_;
};

} // namespace fear