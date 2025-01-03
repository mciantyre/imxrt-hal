//! Demonstrates the secure real-time counter.
//!
//! This example tries to set the SRTC counter value. If the SRTC
//! is already counting (from a previous execution, for instance),
//! then `was_enabled` is `true`, and the SRTC continues counting from
//! its current value. Otherwise, `was_enabled` is `false`, and the
//! SRTC starts counting from a new value.
//!
//! The `was_enabled` state retention depends on how you program your board,
//! and how your board is powered. This specifically includes power to your
//! chip's low-power domains.

#![no_std]
#![no_main]

use hal::snvs::srtc::EnabledState;
use imxrt_hal as hal;

#[imxrt_rt::entry]
fn main() -> ! {
    let (
        board::Common {
            srtc,
            mut snvs_lp_core,
            ..
        },
        board::Specifics { led, .. },
    ) = board::new();

    let (srtc, was_enabled) = match srtc.try_enable(&mut snvs_lp_core, 1600000000, 0) {
        EnabledState::AlreadyCounting { srtc, .. } => (srtc, true),
        EnabledState::SetTime(srtc) => (srtc, false),
    };

    let mut then = 0;
    loop {
        let now = srtc.get();
        if now != then {
            then = now;
            led.toggle();
            defmt::println!("SRTC time: {=u32}. Was enable? {=bool}", now, was_enabled);
        }
    }
}
