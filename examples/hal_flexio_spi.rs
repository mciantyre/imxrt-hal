//! FlexIO example, demonstrating a SPI peripheral.
//!
//! Connect your board's SDI and SDO pins together,
//! forming a loopback. You can then observe 32-bit
//! frames with your scope. Any mismatches are logged.

#![no_std]
#![no_main]

use imxrt_hal::flexio::{
    PinConfig, PinPolarity, PinSelection, Shifter, ShifterConfig, ShifterInput, ShifterMode,
    ShifterStartBit, ShifterStopBit, ShifterTimerPolarity, Timer, TimerConfig, TimerDecrement,
    TimerDisable, TimerEnable, TimerMode, TimerOutput, TimerReset, TimerStartBit, TimerStopBit,
    TimerTrigger, TimerTriggerPolarity,
};

#[imxrt_rt::entry]
fn main() -> ! {
    let (board::Common { .. }, board::Specifics { led, flexio, .. }) = board::new();
    let (mut flexio, [sdo, sck, cs, sdi]) = flexio;

    let sdo_shifter = ShifterConfig {
        mode: ShifterMode::Transmit,
        timer: Timer::Timer0,
        timer_polarity: ShifterTimerPolarity::NegativeEdge,
        pin: PinSelection {
            select: sdo,
            config: PinConfig::Output,
            polarity: PinPolarity::ActiveHigh,
        },
        input_source: ShifterInput::Pin,
        start_bit: ShifterStartBit::DisabledLoadOnEnable,
        stop_bit: ShifterStopBit::Disabled,
        parallel_width: 0,
    };

    let sdi_shifter = ShifterConfig {
        mode: ShifterMode::Receive,
        timer: Timer::Timer0,
        timer_polarity: ShifterTimerPolarity::PositiveEdge,
        pin: PinSelection {
            select: sdi,
            config: PinConfig::OutputDisabled,
            polarity: PinPolarity::ActiveHigh,
        },
        input_source: ShifterInput::Pin,
        start_bit: ShifterStartBit::DisabledLoadOnEnable,
        stop_bit: ShifterStopBit::Disabled,
        parallel_width: 0,
    };

    let data_timer = TimerConfig {
        mode: TimerMode::DualBaudCounter,
        trigger: TimerTrigger::from_shifter(Shifter::Shifter0),
        trigger_polarity: TimerTriggerPolarity::ActiveLow,
        pin: PinSelection {
            select: sck,
            config: PinConfig::Output,
            polarity: PinPolarity::ActiveHigh,
        },
        output: TimerOutput::ZeroNotAffectedByReset,
        decrement: TimerDecrement::FlexIoClockInputTimerOutput,
        reset: TimerReset::Never,
        disable: TimerDisable::TimerCompare,
        enable: TimerEnable::TriggerHigh,
        stop_bit: TimerStopBit::OnTimerDisable,
        start_bit: TimerStartBit::Enabled,
    };

    let cs_timer = TimerConfig {
        mode: TimerMode::SingleCounter,
        trigger: TimerTrigger::from_timer(Timer::Timer0),
        trigger_polarity: TimerTriggerPolarity::ActiveHigh,
        pin: PinSelection {
            select: cs,
            config: PinConfig::Output,
            polarity: PinPolarity::ActiveLow,
        },
        output: TimerOutput::OneNotAffectedByReset,
        decrement: TimerDecrement::FlexIoClockInputTimerOutput,
        reset: TimerReset::Never,
        disable: TimerDisable::PreviousTimerDisable,
        enable: TimerEnable::PreviousTimerEnable,
        stop_bit: TimerStopBit::Disabled,
        start_bit: TimerStartBit::Disabled,
    };

    flexio.configure_shifter(Shifter::Shifter0, &sdo_shifter);
    flexio.configure_shifter(Shifter::Shifter1, &sdi_shifter);
    flexio.configure_timer(Timer::Timer0, &data_timer);
    flexio.configure_timer(Timer::Timer1, &cs_timer);

    // Transfer 32 bit per word. Divide by 4.
    let compare: u16 = ((32 * 2 - 1) << 8) | (4 / 2 - 1);
    flexio.set_timer_compare(Timer::Timer0, compare);

    // CS timer doesn't compare against its value.
    flexio.set_timer_compare(Timer::Timer1, u16::MAX);

    flexio.set_enable(true);

    let mut mismatches = 0_usize;
    let expectations = [
        0,
        u32::MAX,
        0xDEAD_BEEF,
        0xAD1C_AC1D,
        0xAAAA_AAAA,
        0x5555_5555,
    ];

    for expected in expectations.into_iter().cycle() {
        led.toggle();

        while !flexio.shifter_status().contains(Shifter::Shifter0.mask()) {}

        flexio.write_shifter_bit_swapped(Shifter::Shifter0, expected);

        while !flexio.shifter_status().contains(Shifter::Shifter1.mask()) {}

        let recv = flexio.read_shifter_bit_swapped(Shifter::Shifter1);
        if recv != expected {
            mismatches = mismatches.saturating_add(1);
            defmt::println!(
                "{=usize} mismatches, expected {=u32:#010X}!",
                mismatches,
                expected
            );
        }
    }

    defmt::unreachable!();
}
