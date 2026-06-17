// clang-format off
#include <fear.hpp>
// clang-format on

#include <cassert>
#include <fstream>
#include <iostream>

void build(fear::FunctionDef& def)
{
    auto entry = def.entryBlock();

    auto ptr   = def.alloc(fear::Type::Int32);
    auto cond  = def.undef(fear::Type::Bool);

    auto b1    = def.createBlock();
    auto b2    = def.createBlock();
    auto merge = def.createBlock();

    def.jmpif(cond, b1, {}, b2, {});

    def.switchTo(b1);
    def.store(ptr, def.iconst(fear::Type::Int32, 42));
    def.jmp(merge);

    def.switchTo(b2);
    def.store(ptr, def.iconst(fear::Type::Int32, 43));
    def.jmp(merge);

    def.switchTo(merge);
    def.ret(def.load(fear::Type::Int32, ptr));
}

int main()
{
    fear::initLogging();

    fear::Module m("diamond");

    auto fn = fear::Function::declare(&m, "test", {}, fear::Type::Int32);

    fear::FunctionDef def;
    build(def);

    fn.define(def);

    std::cout << "=== Before ===\n";
    std::cout << m.dumpToString() << '\n';

    m.verify();
    m.optimize(fear::OptLevel::Full);

    std::cout << "=== After ===\n";
    std::cout << m.dumpToString() << '\n';

    /// dumps control flow graph
    auto cfg = std::ofstream("diamond.cfg");
    cfg << def.dumpCfgToString();
    cfg.close();

    m.verify();
}