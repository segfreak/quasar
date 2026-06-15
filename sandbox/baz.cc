#include <cstdio>
#include <fear.hpp>
#include <fstream>
#include <optional>

int main()
{
    using namespace fear;

    fearInitLogging();

    Module m("hello");

    auto bar = Function::declare(&m, "bar", {Type::Pointer}, Type::Int32);
    auto baz = Function::declare(&m, "baz", {}, Type::Int32);
    FunctionDef f{};
    auto        slot = f.alloca(Type::Int32);
    f.call(bar.getId(), Type::Void, std::vector<ValueId>{slot});
    auto undef = f.load(Type::Int32, slot);
    f.ret(undef);
    baz.define(f);

    m.optimize(OptLevel::Default);

    std::ofstream out("baz.bin", std::ios::binary);
    auto          buffer = m.binaryDumpToBuffer();
    out.write(reinterpret_cast<const char*>(buffer.data()), buffer.size());
    out.close();

    fprintf(stderr, "=> baz.o\n");
    FILE* file = fopen("baz.o", "w");
    m.emitObject(fear::OptLevel::Full, fileno(file), true, std::nullopt,
                 std::nullopt, Backend::Llvm);
    fclose(file);
}
