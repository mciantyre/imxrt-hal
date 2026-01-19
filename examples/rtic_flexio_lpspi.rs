//! Emulate a SPI controller with FlexIO.
//!
//! The FlexIO peripheral acts as the SPI controller.
//! LPSPI acts as the device. Every read from the device
//! returns an incrementing number.
//!
//! Connect your board's FlexIO signals to your board's
//! LPSPI signals. I/O runs as fast as possible, and you
//! should see around a 600KHz SCK.

#![no_main]
#![no_std]

#[rtic::app(device = board, peripherals = false, dispatchers = [BOARD_SWTASK0])]
mod app {

    use hal::flexio::{
        PinConfig, PinPolarity, PinSelection, Shifter, ShifterConfig, ShifterInput, ShifterMode,
        ShifterStartBit, ShifterStopBit, ShifterTimerPolarity, Timer, TimerConfig, TimerDecrement,
        TimerDisable, TimerEnable, TimerMode, TimerOutput, TimerReset, TimerStartBit, TimerStopBit,
        TimerTrigger, TimerTriggerPolarity,
    };
    use hal::lpspi::{BitOrder, Direction, Interrupts, Transaction};
    use imxrt_hal as hal;
    use rtic_sync::signal::*;

    #[local]
    struct Local {
        spi_event: SignalReader<'static, ()>,
        spi_sig: SignalWriter<'static, ()>,
        flexio: hal::flexio::FlexIo,
    }

    #[shared]
    struct Shared {
        spi: board::Spi,
    }

    type Elem = u16;
    const FRAME_SIZE: u16 = (core::mem::size_of::<Elem>() * 8) as u16;

    const BIT_ORDER: BitOrder = BitOrder::Msb;

    #[init(local = [spi_signal: Signal<()> = Signal::new()])]
    fn init(cx: init::Context) -> (Shared, Local) {
        let (
            _,
            board::Specifics {
                mut spi, flexio, ..
            },
        ) = board::new();
        let (mut flexio, [sdo, sck, cs, sdi]) = flexio;

        let (spi_sig, spi_event) = cx.local.spi_signal.split();

        spi.disabled(|spi| spi.set_peripheral_enable(true));

        // Expect Elem bits per transaction from the FlexIO controller.
        let mut transaction = Transaction::new(FRAME_SIZE).unwrap();
        transaction.set_transmit_data_mask(true);
        transaction.set_bit_order(BIT_ORDER);
        spi.enqueue_transaction(transaction);

        // Interrupt once there's no data in the TX FIFO.
        spi.set_watermark(Direction::Tx, 0);

        // Make sure the LPSPI device task is running!
        lpspi_device::spawn().unwrap();

        //
        // Set up FlexIO as a SPI controller.
        //

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

        flexio.set_shifter_config(Shifter::Shifter0, &sdo_shifter);
        flexio.set_shifter_config(Shifter::Shifter1, &sdi_shifter);
        flexio.set_timer_config(Timer::Timer0, &data_timer);
        flexio.set_timer_config(Timer::Timer1, &cs_timer);

        let compare: u16 = ((FRAME_SIZE * 2 - 1) << 8) | (8 / 2 - 1);
        flexio.set_timer_compare(Timer::Timer0, compare);
        flexio.set_timer_compare(Timer::Timer1, u16::MAX);

        flexio.set_enable(true);

        (
            Shared { spi },
            Local {
                spi_sig,
                spi_event,
                flexio,
            },
        )
    }

    #[idle(local = [flexio])]
    fn idle(cx: idle::Context) -> ! {
        let idle::LocalResources { flexio, .. } = cx.local;

        let mut expected = 0_u16;
        loop {
            expected = expected.wrapping_add(1);

            while !flexio.shifter_status().contains(Shifter::Shifter0.mask()) {}

            // Send an all high bitpattern for "don't care."
            // The transaction forces the SPI read.
            flexio.write_shifter(Shifter::Shifter0, u32::MAX);

            // Wait for the data to arrive in our shifter.
            while !flexio.shifter_status().contains(Shifter::Shifter1.mask()) {}
            let word = flexio.read_shifter_bit_swapped(Shifter::Shifter1);

            defmt::assert!(
                word as u16 == expected,
                "expected {=u16:#06X}, saw {=u32:#010X}!",
                expected,
                word
            );
        }
    }

    /// Keeps the LPSPI FIFO full of new messages to send to
    /// the controller.
    ///
    /// Note that the LPSPI FIFO is larger than what we're using
    /// here. We could saturate the FIFO then reduce the number of
    /// interrupts to activate when half / nearly exhausted.
    #[task(shared = [spi], local = [spi_event], priority = 1)]
    async fn lpspi_device(cx: lpspi_device::Context) -> ! {
        let lpspi_device::LocalResources { spi_event, .. } = cx.local;
        let lpspi_device::SharedResources { mut spi, .. } = cx.shared;

        let mut msg = 0_u16;
        loop {
            msg = msg.wrapping_add(1);

            // Prepare a new transaction that only sends data.
            let mut transaction = Transaction::new(FRAME_SIZE).unwrap();
            transaction.set_receive_data_mask(true);
            transaction.set_bit_order(BIT_ORDER);

            // Enqueue the transaction and the data.
            // Unmask the TX FIFO empty interrupt.
            spi.lock(|spi| {
                spi.enqueue_transaction(transaction);
                spi.enqueue_data(u32::from(msg));
                spi.set_interrupts(Interrupts::TRANSMIT_DATA);
            });

            // Wait for an interrupt to signal that we
            // need to re-fill the FIFO.
            spi_event.wait().await;
        }
    }

    /// Clears the SPI interrupt, and signals the LPSPI device task.
    ///
    /// Technically the whole transaction setup could happen inside the
    /// interrupt handler. We're only separating it out into the async
    /// task for fun!
    #[task(binds=BOARD_SPI, shared = [spi], local = [spi_sig], priority = 2)]
    fn spi_interrupt(cx: spi_interrupt::Context) {
        let spi_interrupt::LocalResources { spi_sig, .. } = cx.local;
        let spi_interrupt::SharedResources { mut spi, .. } = cx.shared;

        spi.lock(|spi| {
            let status = spi.status();
            spi.clear_status(status);
            // Otherwise, the nonzero status re-activates
            // the interrupt.

            spi.set_interrupts(Interrupts::empty());
            // Otherwise, the TX FIFO remains empty, and we'll
            // immediately re-activate.
        });

        spi_sig.write(());
    }
}
