#include <fearc.h>
#include <stdio.h>

void
foo_def (FearFunctionDef *def)
{
  FearValueId arg0       = fearCreateFuncParam (def, FearInt8);
  FearValueId arg1       = fearCreateFuncParam (def, FearInt8);
  FearBlockId entryBlock = fearGetEntryBlock (def);
  FearValueId const24 = fearCreateIntConst (def, entryBlock, FearInt8, 24);
  FearValueId const42 = fearCreateIntConst (def, entryBlock, FearInt8, 42);
  FearValueId tmp0
      = fearCreateAdd (def, entryBlock, FearInt8, const24, const42);
  FearValueId tmp1 = fearCreateAdd (def, entryBlock, FearInt8, arg0, arg1);
  FearValueId tmp  = fearCreateMul (def, entryBlock, FearInt8, tmp0, tmp1);
  fearCreateRet (def, entryBlock, tmp);
}

int
main ()
{
  struct FearModule *m         = fearModuleCreate ("ex");
  FearType           params[2] = { FearInt8, FearInt8 };
  FearFuncId foo = fearDeclareFunction (m, "foo", params, 2, FearInt8,
                                        FearLinkageExternal);

  struct FearFunctionDef *def = fearDefinitionCreate ();
  foo_def (def);
  fearDefineFunction (m, foo, def);

  fprintf (stderr, "preopt\n");
  fearDumpToFile (m, fileno (stderr));

  fearModuleOptimize (m);

  fprintf (stderr, "postopt\n");
  fearDumpToFile (m, fileno (stderr));

  FILE *exf = fopen ("ex.bin", "w");
  fearBinaryDumpToFile (m, fileno (exf));
  fclose (exf);

  fprintf (stderr, "-- features:\n");
  fprintf (stderr, "-- llvm: %d, cranelift: %d\n\n",
           fearLoweringHasLLVM (), fearLoweringHasCranelift ());

  if (fearLoweringHasCranelift ())
  {
    fprintf (stderr, "cranelift: ex.bin -> ex.o\n");

    FILE       *binmodf = fopen ("ex.bin", "r");
    FearModule *exm     = fearReadBinaryFromFile (fileno (binmodf));
    fclose (binmodf);

    FILE *exf_obj = fopen ("ex.o", "w");
    fearEmitCraneliftObjectToFile (m, FearOptLevelFull, fileno (exf_obj));
    fclose (exf_obj);
  }

  return 0;
}
