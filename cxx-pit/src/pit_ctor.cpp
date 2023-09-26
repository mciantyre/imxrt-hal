// Reduce the chance for inlining by defining the ctor in
// a separate module.

#include "cxx-pit/include/pit.hpp"

namespace imxrt {
PitChannel::PitChannel(std::size_t channel) noexcept : channel_{channel} {}
} // namespace imxrt
