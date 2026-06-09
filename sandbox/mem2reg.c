#include <stdint.h>
#include <stdio.h>

#include "fear.h"

int main(void)
{
    FearModule*   m        = fearModuleCreate("mem2reg_stress");

    enum FearType params[] = {FearBool, FearBool, FearBool};

    FearFuncId fid = fearDeclareFunction(m, "test", params, 3, FearInt32,
                                         FearLinkageExternal);

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

    printf("Before optimization:\n");
    char* before = fearDumpToString(m);
    printf("%s\n", before);
    fearStringDispose(before);

    fearModuleOptimize(m, FearOptFull);

    printf("After optimization:\n");
    char* after = fearDumpToString(m);
    printf("%s\n", after);
    fearStringDispose(after);

    fearModuleDispose(m);

    return 0;
}