#include "fear.hpp"
#include <cstdio>
#include <fearc.h>

int
main ()
{
  fear::Module m ("hello");
  auto         mfoo = m.declareFunction ("foo", {}, fear::Type::Int32);
  fear::FunctionDef f{};
  auto slot  = f.createAlloca (f.getEntryBlock (), fear::Type::Int32);
  auto undef = f.createLoad (f.getEntryBlock (), fear::Type::Int32, slot);
  f.createRet (f.getEntryBlock (), undef);
  m.defineFunction (mfoo, f);

  // m.optimize ();
  m.dumpToFile (0);

  FILE *file = fopen ("hello.o", "w");
  m.emitCraneliftObjectToFile (fear::OptLevel::Full, fileno (file));
  fclose (file);
}