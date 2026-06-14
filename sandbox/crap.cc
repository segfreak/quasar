#include <fear.hpp>
#include <fstream>
#include <iostream>

int main()
{
    fear::initLogging();

    fear::Module m("crap");
    auto         f = fear::Function::declare(
        &m, "add", {fear::Type::Int32, fear::Type::Int32},
        fear::Type::Int32);
    f.setCallingConvention(fear::CallConv::C);

    auto def      = fear::FunctionDef();
    auto a        = def.blockParam(fear::Type::Int32);
    auto b        = def.blockParam(fear::Type::Int32);
    auto sum      = def.add(fear::Type::Int32, a, b);
    auto magic    = def.iconst(fear::Type::Int32, 0xDEADBEEF);
    auto xored    = def.bxor(fear::Type::Int32, sum, magic);
    auto mask     = def.iconst(fear::Type::Int32, 0xFFFFFFFF);
    auto masked   = def.band(fear::Type::Int32, xored, mask);
    auto multiply = def.mul(fear::Type::Int32, masked, magic);
    auto divided  = def.div(fear::Type::Int32, multiply, magic);
    def.ret(divided);

    f.define(def);

    if (m.verify() > 0)
    {
        std::cerr << "error: module verification failed" << std::endl;
        return 1;
    }

    std::ofstream out("crap.bin", std::ios::binary);
    auto          buffer = m.binaryDumpToBuffer();
    out.write(reinterpret_cast<const char*>(buffer.data()), buffer.size());
    out.close();

    return 0;
}
