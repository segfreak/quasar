#include <assert.h>
#include <fear.h>

int main()
{
    fearInitLogging();

    FearModule*   m   = fearModuleCreate("dse_test");

    enum FearType i32 = FearInt32;

    FearFuncId    f =
        fearDeclareFunction(m, "test1", &i32, 0, i32, FearLinkageExternal);

    FearFunctionDef* def   = fearDefinitionCreate();

    FearBlockId      entry = fearGetEntryBlock(def);

    FearValueId      p     = fearCreateAlloca(def, entry, i32);

    FearValueId      c1    = fearCreateIntConst(def, entry, i32, 1);
    FearValueId      c2    = fearCreateIntConst(def, entry, i32, 2);
    FearValueId      c3    = fearCreateIntConst(def, entry, i32, 3);

    fearCreateStore(def, entry, p, c1);
    fearCreateStore(def, entry, p, c2);
    fearCreateStore(def, entry, p, c3);

    FearValueId v = fearCreateLoad(def, entry, i32, p);
    fearCreateRet(def, entry, v);

    fearDefineFunction(m, f, def);

    fearModuleOptimize(m, FearOptFull);

    fearModuleVerify(m);

    fearDumpToFile(m, 0);

    fearModuleDispose(m);
    fearDefinitionDispose(def);
}