//! Demonstrates direct usage of the PIT timer.
//!
//! The LED turns on for 250ms, then off for 250ms, repeating.

#![no_std]
#![no_main]

const DELAY_MS: u32 = board::PIT_FREQUENCY / 1_000 * 250;

#[imxrt_rt::entry]
fn main() -> ! {
    let (board::Common { mut pit, .. }, board::Specifics { led, .. }) = board::new();

    pit[0].as_ref().set_load_timer_value(DELAY_MS);
    pit[0].as_mut().enable();

    loop {
        while !pit[0].as_ref().is_elapsed() {}
        pit[0].as_ref().clear_elapsed();
        led.toggle();
    }
}
