#include <umbrella/Logging.hpp>

namespace umbrella
{

std::ostream& errs(std::string_view source)
{
    std::ostream& os = std::cerr;

    os << "umbrella: ";
    if (!source.empty()) { os << source << ": "; }
    os << "error: ";

    return os;
}

std::ostream& dbgs(std::string_view source)
{
    std::ostream& os = std::cerr;

    os << "umbrella: ";
    if (!source.empty()) { os << source << ": "; }
    os << "debug: ";

    return os;
}

}  // namespace umbrella