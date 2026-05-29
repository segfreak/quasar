#include <cstdio>
#include <fear.hpp>

int
main ()
{
  using namespace fear;

  Module      m ("hello");
  auto        foo = Function::declare (&m, "foo", {}, Type::Int32);
  FunctionDef f{};
  auto        slot  = f.stack_alloca (Type::Int32);
  auto        undef = f.load (Type::Int32, slot);
  f.ret (undef);
  foo.define (f);

  m.optimize (OptLevel::Default);
  m.dumpToFile (0);

  FILE *file = fopen ("hello.o", "w");
  m.emitObject (fear::OptLevel::Full, fileno (file));
  fclose (file);
}
