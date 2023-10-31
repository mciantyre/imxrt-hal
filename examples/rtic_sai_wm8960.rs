//! Audio playback using sai peripheral and imxrt-hal.
//!
//! Plays back a simple 440Hz (A note) simple square wave tone with the SAI peripheral
//! to a Wolfson WM8960 codec on a number of the EVK boards.
//!
//! The audio stream itself is expected to be a 48000Hz 16bit stereo signal.

#![no_main]
#![no_std]
#[rtic::app(device = board, peripherals = false, dispatchers = [BOARD_SWTASK0])]
mod app {

    //
    // Configure the demo below.
    //

    /// How frequently (milliseconds) should we poll audio
    const AUDIO_POLL_MS: u32 = board::PIT_FREQUENCY / 1_000 * 250;

    use imxrt_hal as hal;
    //
    // End configurations.
    //

    #[local]
    struct Local {
        /// Toggle when we poll.
        led: board::Led,
        /// This timer tells us how frequently work on audio.
        audio_pit: hal::pit::Pit<2>,
    }

    #[shared]
    struct Shared {}

    #[init]
    fn init(cx: init::Context) -> (Shared, Local, init::Monotonics) {
        let mut cortex_m = cx.core;
        let (
            board::Common {
                pit: (_, _, mut audio_pit, _),
                ..
            },
            board::Specifics { led, .. },
        ) = board::new();
        cortex_m.DCB.enable_trace();
        cortex_m::peripheral::DWT::unlock();
        cortex_m.DWT.enable_cycle_counter();

        audio_pit.set_load_timer_value(AUDIO_POLL_MS);
        audio_pit.set_interrupt_enable(true);
        audio_pit.enable();

        (Shared {}, Local { led, audio_pit }, init::Monotonics())
    }

    #[task(binds = BOARD_PIT, local = [led, audio_pit, counter: u32 = 0], priority = 1)]
    fn pit_interrupt(cx: pit_interrupt::Context) {
        let pit_interrupt::LocalResources {
            audio_pit,
            led,
            counter,
        } = cx.local;

        // Is it time for us to send a new log message?
        if audio_pit.is_elapsed() {
            led.toggle();
            while audio_pit.is_elapsed() {
                audio_pit.clear_elapsed();
            }

            let count = cycles(|| {});

            defmt::println!("Audio synthesis took {=u32} cycles", count);

            *counter += 1;
        }
    }

    /// Count the clock cycles required to execute `f`
    fn cycles<F: FnOnce()>(f: F) -> u32 {
        let start = cortex_m::peripheral::DWT::cycle_count();
        f();
        let end = cortex_m::peripheral::DWT::cycle_count();
        end - start
    }
}
