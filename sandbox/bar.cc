#include <fear.hpp>
#include <fstream>

int main()
{
    using namespace fear;

    fearInitLogging();

    Module m("hello");

    auto bar = Function::declare(&m, "bar", {Type::Pointer}, Type::Int32);
    FunctionDef f{};
    auto        p = f.funcParam(Type::Pointer);
    auto        v = f.load(Type::Int32, p);
    f.ret(v);
    bar.define(f);

    m.optimize(OptLevel::Default);

    std::ofstream out("bar.bin", std::ios::binary);
    auto          buffer = m.binaryDumpToBuffer();
    out.write(reinterpret_cast<const char*>(buffer.data()), buffer.size());
    out.close();
}
