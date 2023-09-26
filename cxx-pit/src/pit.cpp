#include "cxx-pit/include/pit.hpp"

namespace imxrt {
namespace {

struct reg {
  const std::uint32_t address;
  void store(const std::uint32_t value) const noexcept {
    *reinterpret_cast<volatile std::uint32_t *>(address) = value;
  }
  std::uint32_t load() const noexcept {
    return *reinterpret_cast<volatile std::uint32_t *>(address);
  }
};

namespace pit {
constexpr std::uint32_t base = 0x40084000;

constexpr reg mcr{base + 0x00};

constexpr std::uint32_t channel_base(std::size_t channel) noexcept {
  return base + 0x100 + (static_cast<std::uint32_t>(channel) * 0x10);
}

static_assert(channel_base(0) == base + 0x100);
static_assert(channel_base(2) == base + 0x120);

constexpr reg tctrl(std::size_t channel) noexcept {
  return reg{channel_base(channel) + 0x08};
}

static_assert(tctrl(0).address == 0x40084108);

constexpr reg ldval(std::size_t channel) noexcept {
  return reg{channel_base(channel) + 0x00};
}

static_assert(ldval(0).address == 0x40084100);
static_assert(ldval(1).address == 0x40084110);

constexpr reg tflag(std::size_t channel) noexcept {
  return reg{channel_base(channel) + 0x0C};
}

static_assert(tflag(3).address == 0x4008413C);

} // namespace pit
} // namespace

void PitChannel::enable() noexcept { pit::tctrl(channel_).store(1); }

void PitChannel::set_load_timer_value(std::uint32_t ticks) const noexcept {
  pit::ldval(channel_).store(ticks > 0 ? ticks - 1 : 0);
}

bool PitChannel::is_elapsed() const noexcept {
  return pit::tflag(channel_).load() != 0;
}

void PitChannel::clear_elapsed() const noexcept {
  pit::tflag(channel_).store(1);
}

std::array<PitChannel *, 4> initialize_pit() noexcept {
  static std::array<PitChannel, 4> channels{PitChannel{0}, PitChannel{1},
                                            PitChannel{2}, PitChannel{3}};
  pit::mcr.store(0);
  for (std::size_t idx = 0; idx < 4; ++idx)
    pit::tctrl(idx).store(0);

  return std::array<PitChannel *, 4>{
      {&channels[0], &channels[1], &channels[2], &channels[3]}};
}
} // namespace imxrt
