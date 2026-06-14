#include "umbrella/support/Logging.hpp"

#include "umbrella/support/LowercaseStreamBuf.hpp"

namespace umbrella {

std::ostream& errs(std::string_view source)
{
    static LowercaseStreamBuf<char> lowercaseBuf(std::cerr.rdbuf());
    static std::ostream             lowercaseCerr(&lowercaseBuf);

    if (!source.empty()) { lowercaseCerr << source << ": "; }
    lowercaseCerr << "error: ";

    return lowercaseCerr;
}

std::ostream& dbgs(std::string_view source)
{
    static LowercaseStreamBuf<char> lowercaseBuf(std::cerr.rdbuf());
    static std::ostream             lowercaseCerr(&lowercaseBuf);

    if (!source.empty()) { lowercaseCerr << source << ": "; }
    lowercaseCerr << "debug: ";

    return lowercaseCerr;
}

}  // namespace umbrella