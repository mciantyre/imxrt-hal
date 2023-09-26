#pragma once

#include <array>
#include <cstddef>
#include <cstdint>

namespace imxrt {

class PitChannel {
public:
  void set_load_timer_value(std::uint32_t ticks) const noexcept;
  void enable() noexcept;
  bool is_elapsed() const noexcept;
  void clear_elapsed() const noexcept;

  PitChannel(std::size_t channel) noexcept;
  constexpr std::size_t channel() const noexcept { return channel_; }

private:
  std::size_t channel_;
};

std::array<PitChannel *, 4> initialize_pit() noexcept;

} // namespace imxrt
