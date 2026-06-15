#include <fear.hpp>
#include <fstream>

int main()
{
    using namespace fear;

    fearInitLogging();

    Module      m("hello");

    auto        sel = Function::declare(&m, "sel", {}, Type::Int32);
    FunctionDef f;
    auto        a = f.iconst(Type::Int32, 42);
    auto        b = f.iconst(Type::Int32, 56);
    auto        c = f.icmp(IntPredicate::Gt, a, b);
    auto        x = f.select(Type::Int32, c, a, b);
    f.ret(x);
    sel.define(f);

    m.optimize(OptLevel::Default);

    std::ofstream out("sel.bin", std::ios::binary);
    auto          buffer = m.binaryDumpToBuffer();
    out.write(reinterpret_cast<const char*>(buffer.data()), buffer.size());
    out.close();
}
