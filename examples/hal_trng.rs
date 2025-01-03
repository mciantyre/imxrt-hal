//! Demonstrates random numbers from the TRNG.

#![no_std]
#![no_main]

/// How frequently (milliseconds) should we make a random number?
const MAKE_LOG_INTERVAL_MS: u32 = board::PIT_FREQUENCY / 1_000 * 250;

#[imxrt_rt::entry]
fn main() -> ! {
    let (
        board::Common {
            pit: (_, _, mut make_log, _),
            ..
        },
        board::Specifics { led, mut trng, .. },
    ) = board::new();

    make_log.set_load_timer_value(MAKE_LOG_INTERVAL_MS);
    make_log.set_interrupt_enable(false);
    make_log.enable();

    loop {
        if make_log.is_elapsed() {
            led.toggle();
            while make_log.is_elapsed() {
                make_log.clear_elapsed();
            }

            let random = trng.next_u32().unwrap();
            defmt::println!("Random number: {}", random);
        }
    }
}
