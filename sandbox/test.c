#include <fcntl.h>
#include <fear.h>
#include <stdio.h>
#include <unistd.h>

void test_diamond(FearModule *m)
{
    FearType   params[] = {FearInt32, FearInt32, FearInt32};
    FearFuncId fid = fearDeclareFunction(m, "diamond", params, 3,
                                         FearInt32, FearLinkageExternal);

    FearFunctionDef *f     = fearDefinitionCreate();
    FearBlockId      entry = fearGetEntryBlock(f);

    FearBlockId      thenb = fearCreateBlock(f);
    FearBlockId      elseb = fearCreateBlock(f);
    FearBlockId      merge = fearCreateBlock(f);

    FearValueId      a     = fearCreateFuncParam(f, FearInt32);
    FearValueId      b     = fearCreateFuncParam(f, FearInt32);
    FearValueId      c     = fearCreateFuncParam(f, FearInt32);

    FearValueId      zero  = fearCreateIntConst(f, entry, FearInt32, 0);
    FearValueId      cond =
        fearCreateIntCompare(f, entry, FearIntCmpGt, a, zero);

    fearCreateCondJump(f, entry, cond, thenb, NULL, 0, elseb, NULL, 0);

    FearValueId x = fearCreateBlockParam(f, merge, FearInt32);

    fearCreateJump(f, thenb, merge, &b, 1);
    fearCreateJump(f, elseb, merge, &c, 1);

    FearValueId one = fearCreateIntConst(f, merge, FearInt32, 1);
    FearValueId res = fearCreateAdd(f, merge, FearInt32, x, one);
    fearCreateRet(f, merge, res);

    fearDefineFunction(m, fid, f);
}

void test_if_else_chain(FearModule *m)
{
    FearType   params[] = {FearInt32, FearInt32, FearInt32};
    FearFuncId fid = fearDeclareFunction(m, "if_else_chain", params, 3,
                                         FearInt32, FearLinkageExternal);

    FearFunctionDef *f      = fearDefinitionCreate();
    FearBlockId      entry  = fearGetEntryBlock(f);

    FearValueId      a      = fearCreateFuncParam(f, FearInt32);
    FearValueId      b      = fearCreateFuncParam(f, FearInt32);
    FearValueId      c      = fearCreateFuncParam(f, FearInt32);

    FearBlockId      then1  = fearCreateBlock(f);
    FearBlockId      else1  = fearCreateBlock(f);
    FearBlockId      merge1 = fearCreateBlock(f);

    FearBlockId      then2  = fearCreateBlock(f);
    FearBlockId      else2  = fearCreateBlock(f);
    FearBlockId      merge2 = fearCreateBlock(f);

    FearValueId      zero   = fearCreateIntConst(f, entry, FearInt32, 0);
    FearValueId      cond1 =
        fearCreateIntCompare(f, entry, FearIntCmpGt, a, zero);
    fearCreateCondJump(f, entry, cond1, then1, NULL, 0, else1, NULL, 0);

    fearCreateJump(f, then1, merge1, &b, 1);
    fearCreateJump(f, else1, merge1, &c, 1);

    FearValueId x = fearCreateBlockParam(f, merge1, FearInt32);

    FearValueId cond2 =
        fearCreateIntCompare(f, merge1, FearIntCmpGt, x, zero);
    fearCreateCondJump(f, merge1, cond2, then2, NULL, 0, else2, NULL, 0);

    fearCreateJump(f, then2, merge2, &a, 1);
    fearCreateJump(f, else2, merge2, &x, 1);

    FearValueId y   = fearCreateBlockParam(f, merge2, FearInt32);
    FearValueId one = fearCreateIntConst(f, merge2, FearInt32, 1);
    FearValueId res = fearCreateAdd(f, merge2, FearInt32, y, one);
    fearCreateRet(f, merge2, res);

    fearDefineFunction(m, fid, f);
}

void test_early_return(FearModule *m)
{
    FearType   params[] = {FearInt32, FearInt32, FearInt32};
    FearFuncId fid = fearDeclareFunction(m, "early_return", params, 3,
                                         FearInt32, FearLinkageExternal);

    FearFunctionDef *f     = fearDefinitionCreate();
    FearBlockId      entry = fearGetEntryBlock(f);

    FearValueId      a     = fearCreateFuncParam(f, FearInt32);
    FearValueId      b     = fearCreateFuncParam(f, FearInt32);
    FearValueId      c     = fearCreateFuncParam(f, FearInt32);

    FearBlockId      thenb = fearCreateBlock(f);
    FearBlockId      elseb = fearCreateBlock(f);
    FearBlockId      merge = fearCreateBlock(f);

    FearValueId      zero  = fearCreateIntConst(f, entry, FearInt32, 0);
    FearValueId      cond =
        fearCreateIntCompare(f, entry, FearIntCmpGt, a, zero);
    fearCreateCondJump(f, entry, cond, thenb, NULL, 0, elseb, NULL, 0);

    // early return
    fearCreateRet(f, thenb, b);

    fearCreateJump(f, elseb, merge, &c, 1);

    FearValueId x   = fearCreateBlockParam(f, merge, FearInt32);
    FearValueId res = fearCreateAdd(f, merge, FearInt32, x, zero);
    fearCreateRet(f, merge, res);

    fearDefineFunction(m, fid, f);
}

void test_cross_phi(FearModule *m)
{
    FearType   params[] = {FearInt32, FearInt32, FearInt32};
    FearFuncId fid = fearDeclareFunction(m, "cross_phi", params, 3,
                                         FearInt32, FearLinkageExternal);

    FearFunctionDef *f     = fearDefinitionCreate();
    FearBlockId      entry = fearGetEntryBlock(f);

    FearValueId      a     = fearCreateFuncParam(f, FearInt32);
    FearValueId      b     = fearCreateFuncParam(f, FearInt32);
    FearValueId      c     = fearCreateFuncParam(f, FearInt32);

    FearBlockId      thenb = fearCreateBlock(f);
    FearBlockId      elseb = fearCreateBlock(f);
    FearBlockId      merge = fearCreateBlock(f);

    FearValueId      zero  = fearCreateIntConst(f, entry, FearInt32, 0);
    FearValueId      sum   = fearCreateAdd(f, entry, FearInt32, a, b);
    FearValueId      cond =
        fearCreateIntCompare(f, entry, FearIntCmpGt, sum, zero);

    fearCreateCondJump(f, entry, cond, thenb, NULL, 0, elseb, NULL, 0);

    FearValueId t1 = fearCreateAdd(f, thenb, FearInt32, b, c);
    fearCreateJump(f, thenb, merge, &t1, 1);

    FearValueId t2 = fearCreateSub(f, elseb, FearInt32, b, c);
    fearCreateJump(f, elseb, merge, &t2, 1);

    FearValueId x = fearCreateBlockParam(f, merge, FearInt32);
    fearCreateRet(f, merge, x);

    fearDefineFunction(m, fid, f);
}

void test_big_cfg_stress(FearModule *m)
{
    FearType   params[] = {FearInt32, FearInt32, FearInt32};
    FearFuncId fid = fearDeclareFunction(m, "big_cfg_stress", params, 3,
                                         FearInt32, FearLinkageExternal);

    FearFunctionDef *f     = fearDefinitionCreate();
    FearBlockId      entry = fearGetEntryBlock(f);

    FearValueId      a     = fearCreateFuncParam(f, FearInt32);
    FearValueId      b     = fearCreateFuncParam(f, FearInt32);
    FearValueId      c     = fearCreateFuncParam(f, FearInt32);

    FearBlockId      t1    = fearCreateBlock(f);
    FearBlockId      e1    = fearCreateBlock(f);
    FearBlockId      m1    = fearCreateBlock(f);

    FearBlockId      t2    = fearCreateBlock(f);
    FearBlockId      e2    = fearCreateBlock(f);
    FearBlockId      m2    = fearCreateBlock(f);

    FearBlockId      t3    = fearCreateBlock(f);
    FearBlockId      e3    = fearCreateBlock(f);
    FearBlockId      m3    = fearCreateBlock(f);

    FearValueId      zero  = fearCreateIntConst(f, entry, FearInt32, 0);
    FearValueId      cond1 =
        fearCreateIntCompare(f, entry, FearIntCmpGt, a, zero);
    fearCreateCondJump(f, entry, cond1, t1, NULL, 0, e1, NULL, 0);

    fearCreateJump(f, t1, m1, &b, 1);
    fearCreateJump(f, e1, m1, &c, 1);

    FearValueId x1   = fearCreateBlockParam(f, m1, FearInt32);
    FearValueId tmp1 = fearCreateAdd(f, m1, FearInt32, x1, a);
    FearValueId tmp2 =
        fearCreateSub(f, m1, FearInt32, tmp1, a); // dead after CSE

    FearValueId cond2 =
        fearCreateIntCompare(f, m1, FearIntCmpGt, tmp2, zero);
    fearCreateCondJump(f, m1, cond2, t2, NULL, 0, e2, NULL, 0);

    fearCreateJump(f, t2, m2, &tmp1, 1);
    fearCreateJump(f, e2, m2, &x1, 1);

    FearValueId x2   = fearCreateBlockParam(f, m2, FearInt32);
    FearValueId tmp3 = fearCreateMul(
        f, m2, FearInt32, x2, fearCreateIntConst(f, m2, FearInt32, 2));
    FearValueId tmp4 =
        fearCreateDiv(f, m2, FearInt32, tmp3,
                      fearCreateIntConst(f, m2, FearInt32, 2)); // cancels

    FearValueId cond3 =
        fearCreateIntCompare(f, m2, FearIntCmpGt, tmp4, zero);
    fearCreateCondJump(f, m2, cond3, t3, NULL, 0, e3, NULL, 0);

    fearCreateJump(f, t3, m3, &tmp4, 1);
    fearCreateJump(f, e3, m3, &x2, 1);

    FearValueId x3  = fearCreateBlockParam(f, m3, FearInt32);
    FearValueId res = fearCreateAdd(
        f, m3, FearInt32, x3, fearCreateIntConst(f, m3, FearInt32, 1));
    fearCreateRet(f, m3, res);

    fearDefineFunction(m, fid, f);
}

void test_memory_stack_heavy(FearModule *m)
{
    FearType   params[] = {FearInt32, FearInt32, FearInt32};
    FearFuncId fid =
        fearDeclareFunction(m, "memory_stack_heavy", params, 3, FearInt32,
                            FearLinkageExternal);

    FearFunctionDef *f       = fearDefinitionCreate();
    FearBlockId      entry   = fearGetEntryBlock(f);

    FearValueId      zero32  = fearCreateIntConst(f, entry, FearInt32, 0);

    // function args
    FearValueId      a       = fearCreateFuncParam(f, FearInt32);
    FearValueId      b       = fearCreateFuncParam(f, FearInt32);
    FearValueId      c       = fearCreateFuncParam(f, FearInt32);

    // stack allocations
    FearValueId      var_ptr = fearCreateAlloca(f, entry, FearInt32);
    FearValueId arr_ptr = fearCreateArrayAlloca(f, entry, FearInt32, 16);

    // store initial value
    fearCreateStore(f, entry, var_ptr, a);

    // blocks
    FearBlockId loop          = fearCreateBlock(f);
    FearBlockId body          = fearCreateBlock(f);
    FearBlockId exit          = fearCreateBlock(f);

    // jump to loop
    FearValueId init_params[] = {zero32, a};
    fearCreateJump(f, entry, loop, init_params, 2);

    // loop phis
    FearValueId i       = fearCreateBlockParam(f, loop, FearInt32);
    FearValueId acc     = fearCreateBlockParam(f, loop, FearInt32);

    FearValueId sixteen = fearCreateIntConst(f, loop, FearInt32, 16);
    FearValueId cond =
        fearCreateIntCompare(f, loop, FearIntCmpLt, i, sixteen);
    fearCreateCondJump(f, loop, cond, body, NULL, 0, exit, NULL, 0);

    // compute &arr[i]
    FearValueId elem_ptr =
        fearCreateElementPtr(f, body, FearInt32, arr_ptr, i);

    // redundant math
    FearValueId t1 = fearCreateAdd(f, body, FearInt32, b, c);
    FearValueId t2 = fearCreateSub(f, body, FearInt32, t1, c); // == b

    // store to array
    fearCreateStore(f, body, elem_ptr, t2);

    // load from array (should CSE with stored value)
    FearValueId loaded  = fearCreateLoad(f, body, FearInt32, elem_ptr);

    // load var_ptr (loop invariant)
    FearValueId base    = fearCreateLoad(f, body, FearInt32, var_ptr);

    FearValueId new_acc = fearCreateAdd(f, body, FearInt32, acc, loaded);
    FearValueId new_acc2 =
        fearCreateAdd(f, body, FearInt32, new_acc, base);

    FearValueId one          = fearCreateIntConst(f, body, FearInt32, 1);
    FearValueId next_i       = fearCreateAdd(f, body, FearInt32, i, one);

    FearValueId tmp_params[] = {next_i, new_acc2};
    fearCreateJump(f, body, loop, tmp_params, 2);

    // exit
    FearValueId final =
        fearCreateAdd(f, exit, FearInt32, acc,
                      fearCreateLoad(f, exit, FearInt32, var_ptr));
    fearCreateRet(f, exit, final);

    fearDefineFunction(m, fid, f);
}

void test_algebraic_simplification(FearModule *m)
{
    FearType   params[] = {FearInt32, FearInt32, FearInt32};
    FearFuncId fid =
        fearDeclareFunction(m, "algebraic_simplification", params, 3,
                            FearInt32, FearLinkageExternal);

    FearFunctionDef *f     = fearDefinitionCreate();
    FearBlockId      entry = fearGetEntryBlock(f);

    FearValueId      a     = fearCreateFuncParam(f, FearInt32);
    FearValueId      b     = fearCreateFuncParam(f, FearInt32);
    FearValueId      c     = fearCreateFuncParam(f, FearInt32);

    FearValueId      zero  = fearCreateIntConst(f, entry, FearInt32, 0);
    FearValueId      one   = fearCreateIntConst(f, entry, FearInt32, 1);
    FearValueId      eight = fearCreateIntConst(f, entry, FearInt32, 8);
    FearValueId      nine  = fearCreateIntConst(f, entry, FearInt32, 9);
    FearValueId      ten   = fearCreateIntConst(f, entry, FearInt32, 10);

    FearValueId b_mul_zero = fearCreateMul(f, entry, FearInt32, b, zero);
    FearValueId dead_zero =
        fearCreateAdd(f, entry, FearInt32, b_mul_zero, zero);

    FearValueId self_sub = fearCreateSub(f, entry, FearInt32, b, b);

    FearValueId t1       = fearCreateAdd(f, entry, FearInt32, a, ten);
    FearValueId t2       = fearCreateSub(f, entry, FearInt32, t1, ten);
    FearValueId t3       = fearCreateMul(f, entry, FearInt32, one, c);
    FearValueId t4       = fearCreateMul(f, entry, FearInt32, t2, nine);
    FearValueId t5       = fearCreateDiv(f, entry, FearInt32, t3, eight);

    FearValueId sum1     = fearCreateAdd(f, entry, FearInt32, t4, t5);
    FearValueId sum2 = fearCreateAdd(f, entry, FearInt32, sum1, dead_zero);
    FearValueId res  = fearCreateAdd(f, entry, FearInt32, sum2, self_sub);

    fearCreateRet(f, entry, res);
    fearDefineFunction(m, fid, f);
}

int main()
{
    fearInitialiseLogging();

    struct FearModule *mod = fearModuleCreate("faz");
    if (!mod)
    {
        fprintf(stderr, "cannot create module\n");
        return 1;
    }

    test_diamond(mod);
    test_if_else_chain(mod);
    test_early_return(mod);
    test_cross_phi(mod);
    test_big_cfg_stress(mod);
    test_memory_stack_heavy(mod);
    test_algebraic_simplification(mod);

    fearModuleVerify(mod);

#ifdef MULT
    uint32_t passes = fearModuleOptimizeMultilevel(mod, FearOptDefault);
#else
    uint32_t passes = fearModuleOptimize(mod, FearOptDefault);
#endif

    fearDumpToFile(mod, 0);

    FearBackend backend;
    if ((backend = fearSelectBackendForObject()) &&
        fearHasBackend(backend))
    {
        fprintf(stderr, "=> test.o\n");
        FILE *f = fopen("test.o", "w");
        fearEmitObject(mod, backend, FearOptFull, fileno(f));
        fclose(f);
    }

    uint32_t errors = fearModuleVerify(mod);

    printf("passes: %u\n", passes);
    printf("errors: %u\n", errors);

    fearModuleDispose(mod);
    return 0;
}