#pragma once

#include <iostream>
#include <ostream>
#include <string_view>

namespace umbrella
{

std::ostream& errs(std::string_view source = "");
std::ostream& dbgs(std::string_view source = "");

}  // namespace umbrella
