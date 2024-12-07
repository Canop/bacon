#include <iostream>

void warnings() {
    auto unused_var = 0;
    1 + 2;
}

auto main() -> int {
    std::cout << "standard message here\n";
    return 0;
}
