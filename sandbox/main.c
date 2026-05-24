#include <fearc.h>
#include <stdio.h>

void
foo_def (FearFunctionDef *def)
{
  FearValueId arg0       = fearCreateFuncParam (def, FearInt64);
  FearBlockId entryBlock = fearGetEntryBlock (def);

  FearValueId slot = fearCreateArrayAlloca (def, entryBlock, FearInt64, 2);

  FearValueId zero = fearCreateIntConst (def, entryBlock, FearInt32, 0);
  FearValueId one  = fearCreateIntConst (def, entryBlock, FearInt32, 1);

  FearValueId first_addr
      = fearCreateElementPtr (def, entryBlock, FearInt32, slot, zero);
  FearValueId second_addr
      = fearCreateElementPtr (def, entryBlock, FearInt32, slot, one);

  fearCreateStore (def, entryBlock, first_addr, zero);
  fearCreateStore (def, entryBlock, second_addr, one);

  FearValueId first
      = fearCreateLoad (def, entryBlock, FearInt32, first_addr);
  FearValueId second
      = fearCreateLoad (def, entryBlock, FearInt32, second_addr);

  FearValueId sum
      = fearCreateAdd (def, entryBlock, FearInt32, first, second);

  fearCreateRet (def, entryBlock, sum);
}

int
main ()
{
  struct FearModule *m         = fearModuleCreate ("ex");
  FearType           params[1] = { FearInt32 };
  FearFuncId foo = fearDeclareFunction (m, "foo", params, 1, FearInt32,
                                        FearLinkageExternal);

  struct FearFunctionDef *def = fearDefinitionCreate ();
  foo_def (def);
  fearDefineFunction (m, foo, def);
  fearDefinitionDispose (def);

  fprintf (stderr, "preopt\n");
  fearDumpToFile (m, fileno (stderr));

  unsigned total_passes = fearModuleOptimize (m);

  fprintf (stderr, "postopt (%d passes)\n", total_passes);
  fearDumpToFile (m, fileno (stderr));

  FILE *exf = fopen ("ex.bin", "w");
  fearBinaryDumpToFile (m, fileno (exf));
  fclose (exf);

  fearModuleDispose (m);

  fprintf (stderr, "-- features:\n");
  fprintf (stderr, "-- llvm: %d, cranelift: %d\n\n",
           fearHasBackend (FearBackendLlvm),
           fearHasBackend (FearBackendCranelift));

  if (fearHasBackend (FearBackendLlvm))
  {
    fearEmitAssembly (m, FearBackendLlvm, FearOptLevelFull, 1);
  }

  if (fearHasBackend (FearBackendCranelift))
  {
    fprintf (stderr, "=> ex.o\n");

    FILE *exf_obj = fopen ("ex.o", "w");
    fearEmitObject (m, FearBackendCranelift, FearOptLevelFull,
                    fileno (exf_obj));
    fclose (exf_obj);
  }

  return 0;
}
