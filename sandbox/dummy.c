#include <fear.h>

/// memory leak here
/// idk..
int main(void)
{
    FearModule* m  = fearModuleCreate("dummy");

    FearFuncId  id = fearDeclareFunction(m, "idk", NULL, 0, FearInt8,
                                         FearLinkageInternal);

    fearModuleDispose(m);
}