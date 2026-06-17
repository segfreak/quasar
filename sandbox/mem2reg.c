#include <getopt.h>
#include <stdint.h>
#include <stdio.h>

#include "fear.h"

void emit(int do_opt, const char* triple)
{
    fearInitLogging();

    printf("triple: %s\n", triple);

    FearModule*   m        = fearModuleCreate("mem2reg");

    enum FearType params[] = {FearBool, FearBool, FearBool};

    FearFuncId fid = fearDeclareFunction(m, "mem2reg", params, 3,
                                         FearInt32, FearLinkageExternal);

    FearFunctionDef* f        = fearDefinitionCreate();

    FearBlockId      entry    = fearGetEntryBlock(f);

    FearValueId      a        = fearCreateFuncParam(f, FearBool);
    FearValueId      b        = fearCreateFuncParam(f, FearBool);
    FearValueId      c        = fearCreateFuncParam(f, FearBool);

    FearBlockId      then_a   = fearCreateBlock(f);
    FearBlockId      else_a   = fearCreateBlock(f);

    FearBlockId      then_b   = fearCreateBlock(f);
    FearBlockId      else_b   = fearCreateBlock(f);

    FearBlockId      merge_ab = fearCreateBlock(f);

    FearBlockId      then_c   = fearCreateBlock(f);
    FearBlockId      merge_c  = fearCreateBlock(f);

    FearValueId      x        = fearCreateAlloca(f, entry, FearInt32);

    FearValueId      zero     = fearCreateIntConst(f, entry, FearInt32, 0);
    fearCreateStore(f, entry, x, zero);

    fearCreateCondJump(f, entry, a, then_a, NULL, 0, else_a, NULL, 0);

    /*
        if (a)
    */

    fearCreateCondJump(f, then_a, b, then_b, NULL, 0, else_b, NULL, 0);

    /*
        x = 1
    */

    FearValueId one = fearCreateIntConst(f, then_b, FearInt32, 1);

    fearCreateStore(f, then_b, x, one);

    fearCreateJump(f, then_b, merge_ab, NULL, 0);

    /*
        x = 2
    */

    FearValueId two = fearCreateIntConst(f, else_b, FearInt32, 2);

    fearCreateStore(f, else_b, x, two);

    fearCreateJump(f, else_b, merge_ab, NULL, 0);

    /*
        else branch of a
    */

    fearCreateJump(f, else_a, merge_ab, NULL, 0);

    /*
        if (c)
    */

    fearCreateCondJump(f, merge_ab, c, then_c, NULL, 0, merge_c, NULL, 0);

    /*
        x = 3
    */

    FearValueId three = fearCreateIntConst(f, then_c, FearInt32, 3);

    fearCreateStore(f, then_c, x, three);

    fearCreateJump(f, then_c, merge_c, NULL, 0);

    /*
        return x
    */

    FearValueId result = fearCreateLoad(f, merge_c, FearInt32, x);

    fearCreateRet(f, merge_c, result);

    fearDefineFunction(m, fid, f);

    fearDefinitionDispose(f);

    printf("noopt\n");
    char* before = fearDumpToString(m);
    printf("%s\n", before);
    fearStringDispose(before);

    if (do_opt)
    {
        fearModuleOptimize(m, FearOptFull);
        printf("opt:\n");
        char* after = fearDumpToString(m);
        printf("%s\n", after);
        fearStringDispose(after);
    }

    FILE* out = fopen("mem2reg.bin", "wb");
    fearBinaryDumpToFile(m, out);
    fclose(out);

    FearBackend backend;
    if ((backend = fearSelectBackendForObject()) &&
        fearHasBackend(backend))
    {
        fprintf(stderr, "=> mem2reg.o\n");
        FILE* exf_obj = fopen("mem2reg.o", "w");
        fearEmitObject(m, backend, FearOptFull, 1, triple, NULL, exf_obj);
        fclose(exf_obj);
    }

    fearModuleDispose(m);
}

int main(int argc, char** argv)
{
    int                  opt;

    int                  do_opt         = 0;
    char*                triple         = NULL;

    static struct option long_options[] = {
        {"triple", required_argument, 0, 't'},
        {   "opt",       no_argument, 0, 'o'},
        {       0,                 0, 0,   0}
    };

    while ((opt = getopt_long(argc, argv, "t:o", long_options, NULL)) !=
           -1)
    {
        switch (opt)
        {
            case 't':
                triple = optarg;
                break;
            case 'o':
                do_opt = 1;
                break;
            default:
                fprintf(stderr, "usage: %s [--triple TRIPLE] [--opt]\n",
                        argv[0]);
                exit(EXIT_FAILURE);
        }
    }

    emit(do_opt, triple);
}