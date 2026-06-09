#include <assert.h>
#include <fear.h>
#include <stdio.h>

int main()
{
    fearInitLogging();

    FearModule*      m   = fearModuleCreate("global_dse");

    enum FearType    i32 = FearInt32;

    FearFuncId       f = fearDeclareFunction(m, "global_dse", &i32, 1, i32,
                                             FearLinkageExternal);

    FearFunctionDef* def   = fearDefinitionCreate();

    FearBlockId      B0    = fearGetEntryBlock(def);
    FearBlockId      B1    = fearCreateBlock(def);
    FearBlockId      B2    = fearCreateBlock(def);
    FearBlockId      B3    = fearCreateBlock(def);

    // function parameter (i32 c)
    FearBlockId      param = fearCreateFuncParam(def, i32);

    // stack slot
    FearValueId      p     = fearCreateAlloca(def, B0, i32);

    FearValueId      c1    = fearCreateIntConst(def, B0, i32, 10);
    FearValueId      c2    = fearCreateIntConst(def, B0, i32, 20);
    FearValueId      c3    = fearCreateIntConst(def, B0, i32, 30);

    // B0 store
    fearCreateStore(def, B0, p, c1);

    // fake condition
    FearValueId cond =
        fearCreateIntCompare(def, B0, FearIntCmpEq, param, param);

    FearValueId empty1[0];
    FearValueId empty2[0];

    // branch
    fearCreateCondJump(def, B0, cond, B1, empty1, 0, B2, empty2, 0);

    // B1 store
    fearCreateStore(def, B1, p, c2);
    fearCreateJump(def, B1, B3, NULL, 0);

    // B2 store
    fearCreateStore(def, B2, p, c3);
    fearCreateJump(def, B2, B3, NULL, 0);

    // B3 load + return
    FearValueId v = fearCreateLoad(def, B3, i32, p);
    fearCreateRet(def, B3, v);

    fearSetEntryBlock(def, B0);

    fearDefineFunction(m, f, def);

    printf("Before optimization:\n");
    char* before = fearDumpToString(m);
    printf("%s\n", before);
    fearStringDispose(before);

    fearModuleOptimize(m, FearOptFull);

    printf("After optimization:\n");
    char* after = fearDumpToString(m);
    printf("%s\n", after);
    fearStringDispose(after);

    fearModuleVerify(m);

    fearModuleDispose(m);
    fearDefinitionDispose(def);

    return 0;
}