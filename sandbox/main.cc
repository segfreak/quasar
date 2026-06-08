#include <cstdio>
#include <fear.hpp>

int main()
{
    using namespace fear;

    fearInitLogging();

    Module m("hello");

    auto   bar = Function::declare(&m, "bar", {Type::Pointer}, Type::Void);

    auto   foo = Function::declare(&m, "foo", {}, Type::Int32);
    FunctionDef f{};
    auto        slot  = f.stack_alloca(Type::Int32);
    // f.call(bar.getId(), Type::Void, std::vector<ValueId>{slot});
    auto        undef = f.load(Type::Int32, slot);
    f.ret(undef);
    foo.define(f);

    m.optimize(OptLevel::Default);
    m.dumpToFile(0);

    fprintf(stderr, "=> tmaincc.o\n");
    FILE* file = fopen("tmaincc.o", "w");
    m.emitObject(fear::OptLevel::Full, fileno(file), Backend::Llvm);
    fclose(file);
}
