#include <cassert>
#include <fear.hpp>
#include <iostream>

int main()
{
    fear::initLogging();

    fear::Module m("gvn");

    auto         fn = fear::Function::declare(
        &m, "test",
        {fear::Type::Int32, fear::Type::Int32, fear::Type::Bool},
        fear::Type::Int32);

    fear::FunctionDef def;

    auto              a     = def.funcParam(fear::Type::Int32);
    auto              b     = def.funcParam(fear::Type::Int32);
    auto              c     = def.funcParam(fear::Type::Bool);

    auto              entry = def.entryBlock();
    auto              bb1   = def.createBlock();
    auto              bb2   = def.createBlock();

    def.jmpif(c, bb1, {}, bb2, {});

    def.switchTo(bb1);

    auto y = def.add(fear::Type::Int32, a, b);
    def.ret(y);

    def.switchTo(bb2);

    auto y2 = def.add(fear::Type::Int32, a, b);
    def.ret(y2);

    fn.define(def);

    std::cout << "=== Before ===\n";
    std::cout << m.dumpToString() << '\n';

    m.verify();
    m.optimize(fear::OptLevel::Full);

    std::cout << "=== After ===\n";
    std::cout << m.dumpToString() << '\n';

    m.verify();
}