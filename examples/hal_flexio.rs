//! FlexIO example, demonstrating a SPI peripheral.
//!
//! For simplicity, this doesn't set up a data input signal,
//! SDI. It only demonstrates the SDO, SCK, and CS functions.

#![no_std]
#![no_main]

use hal::gpt::OutputCompareRegister::OCR1 as OCR;
use imxrt_hal as hal;
use imxrt_hal::flexio::{
    PinConfig, PinPolarity, PinSelection, Shifter, ShifterConfig, ShifterInput, ShifterMode,
    ShifterStartBit, ShifterStopBit, ShifterTimerPolarity, Timer, TimerConfig, TimerDecrement,
    TimerDisable, TimerEnable, TimerMode, TimerOutput, TimerReset, TimerStartBit, TimerStopBit,
    TimerTriggerPolarity, TimerTriggerSelect, TimerTriggerSource,
};

const GPT1_DELAY_MS: u32 = board::GPT1_FREQUENCY / 1_000 / 1_000 * 250;

#[imxrt_rt::entry]
fn main() -> ! {
    let (board::Common { mut gpt1, .. }, board::Specifics { led, flexio, .. }) = board::new();
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
        parallel_width: 0, // 1-bit serial
    };

    let sdi_shifter = ShifterConfig {
        mode: ShifterMode::Receive,
        timer: Timer::Timer0,
        timer_polarity: ShifterTimerPolarity::PositiveEdge,
        pin: PinSelection {
            select: sdi,
            config: PinConfig::Disabled,
            polarity: PinPolarity::ActiveHigh,
        },
        input_source: ShifterInput::Pin,
        start_bit: ShifterStartBit::DisabledLoadOnEnable,
        stop_bit: ShifterStopBit::Disabled,
        parallel_width: 0,
    };

    let data_timer = TimerConfig {
        mode: TimerMode::DualBaudCounter,
        trigger_source: TimerTriggerSource::Internal,
        trigger_polarity: TimerTriggerPolarity::ActiveLow,
        trigger_select: TimerTriggerSelect::from_shifter(Shifter::Shifter0),
        pin: PinSelection {
            select: sck,
            config: PinConfig::Output,
            polarity: PinPolarity::ActiveHigh,
        },
        output: TimerOutput::ZeroNotAffectedByReset,
        decrement: TimerDecrement::FlexIoClockShiftOnOutput,
        reset: TimerReset::Never,
        disable: TimerDisable::TimerCompare, // Turn off until the next shift buffer write.
        enable: TimerEnable::TriggerHigh,    // Enable when shifter has data
        stop_bit: TimerStopBit::EnabledOnTimerDisable,
        start_bit: TimerStartBit::Enabled,
    };

    let cs_timer = TimerConfig {
        mode: TimerMode::SingleCounter,
        trigger_select: TimerTriggerSelect::from_timer(Timer::Timer0),
        trigger_polarity: TimerTriggerPolarity::ActiveHigh,
        trigger_source: TimerTriggerSource::Internal,
        pin: PinSelection {
            select: cs,
            config: PinConfig::Output,
            polarity: PinPolarity::ActiveLow,
        },
        output: TimerOutput::OneNotAffectedByReset,
        decrement: TimerDecrement::FlexIoClockShiftOnOutput,
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

    gpt1.set_output_compare_count(OCR, GPT1_DELAY_MS);
    gpt1.set_mode(hal::gpt::Mode::Restart);
    gpt1.enable();

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
        gpt1.clear_elapsed(OCR);

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
