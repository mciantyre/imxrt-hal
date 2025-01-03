//! Demonstrates the temperature monitor on 10xx MCUs.
//!
//! This example uses the logging system to relay temperature
//! measurements.

#![no_std]
#![no_main]

/// How frequently (milliseconds) should we make a log message?
///
/// Decrease this constant to log more frequently.
const MAKE_LOG_INTERVAL_MS: u32 = board::PIT_FREQUENCY / 1_000 * 250;

#[imxrt_rt::entry]
fn main() -> ! {
    let (
        board::Common {
            pit: (_, _, mut make_log, _),
            ..
        },
        board::Specifics {
            led, mut tempmon, ..
        },
    ) = board::new();

    // When should we generate a log message?
    make_log.set_load_timer_value(MAKE_LOG_INTERVAL_MS);
    make_log.set_interrupt_enable(false);
    make_log.enable();

    tempmon.start().ok();
    loop {
        if make_log.is_elapsed() {
            led.toggle();
            while make_log.is_elapsed() {
                make_log.clear_elapsed();
            }

            if let Ok(temperature) = tempmon.get_temp() {
                defmt::println!("Temperature (mC'): {=i32}", temperature);
            }
        }
    }
}
