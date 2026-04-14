//! Demonstrates a SPI device with blocking I/O.
//!
//! Connect SDI to SDO. The example uses the LPSPI interrupt to
//! schedule transfers, and to receive data. You can observe the
//! I/O with a scope / logic analyzer. The SPI CLK runs at 1MHz.
//!
//! Keep an eye on the defmt log to see if tests fail.

#![no_std]
#![no_main]

#[rtic::app(device = board, peripherals = false)]
mod app {

    use eh1::spi::SpiBus;
    use imxrt_hal as hal;
    use imxrt_hal::pit::Channel;

    const PIT_CHANNEL: Channel = Channel::Chan2;
    const PIT_DELAY_MS: u32 = board::PIT_FREQUENCY / 1_000 * 250;

    #[local]
    struct Local {
        spi: board::Spi,
        pit: hal::pit::Pit,
    }

    #[shared]
    struct Shared {}

    #[init]
    fn init(_: init::Context) -> (Shared, Local) {
        let (board::Common { pit, .. }, board::Specifics { spi, .. }) = board::new();
        (Shared {}, Local { spi, pit })
    }

    #[idle(local = [spi, pit])]
    fn idle(cx: idle::Context) -> ! {
        let idle::LocalResources { spi, pit, .. } = cx.local;
        pit.set_load_timer_value(PIT_CHANNEL, PIT_DELAY_MS);

        let mut delay = move || {
            pit.enable(PIT_CHANNEL);
            while !pit.is_elapsed(PIT_CHANNEL) {}
            pit.clear_elapsed(PIT_CHANNEL);
            pit.disable(PIT_CHANNEL);
        };

        loop {
            for _ in 0..3 {
                delay();
            }

            // For studying the effects of bit order and word size.
            //
            // If you have a logic analyzer that can change its word
            // size and bit order, use this sequence to evaluate how
            // the driver packs your transfer elements.
            {
                use hal::lpspi::BitOrder::{self, *};

                const BIT_ORDERS: [BitOrder; 2] = [Msb, Lsb];

                const U32_WORDS: [u32; 2] = [0xDEADBEEFu32, 0xAD1CAC1D];
                for bit_order in BIT_ORDERS {
                    spi.set_bit_order(bit_order);
                    spi.write(&U32_WORDS).unwrap();
                }

                const U8_WORDS: [u8; 7] = [0xDEu8, 0xAD, 0xBE, 0xEF, 0x12, 0x34, 0x56];
                for bit_order in BIT_ORDERS {
                    spi.set_bit_order(bit_order);
                    spi.write(&U8_WORDS).unwrap();
                }

                const U16_WORDS: [u16; 3] = [0xDEADu16, 0xBEEF, 0x1234];
                for bit_order in BIT_ORDERS {
                    spi.set_bit_order(bit_order);
                    spi.write(&U16_WORDS).unwrap();
                }

                delay();
            }

            // Change me to explore bit order behavors in the
            // remaining write / loopback transfer tests.
            spi.set_bit_order(hal::lpspi::BitOrder::Msb);

            // Make sure concatenated elements look correct on the wire.
            // Make sure we can read those elements.
            {
                use hal::lpspi::BitOrder;

                macro_rules! transfer_test {
                    ($arr:expr, $bit_order:expr) => {
                        let bit_order_name = match $bit_order {
                            BitOrder::Msb => "MSB",
                            BitOrder::Lsb => "LSB",
                        };

                        spi.set_bit_order($bit_order);
                        let mut buffer = $arr;
                        spi.transfer_in_place(&mut buffer).unwrap();
                        defmt::assert_eq!(buffer, $arr, "In place, bit order {}", bit_order_name);

                        let mut input = [0; $arr.len()];
                        let output = $arr;

                        spi.transfer(&mut input, &output).unwrap();
                        defmt::assert_eq!(
                            input,
                            output,
                            "Eq len transfer, bit order {}",
                            bit_order_name
                        );
                        input.fill(0);

                        spi.transfer(&mut input[1..], &output).unwrap();
                        defmt::assert_eq!(
                            &input[1..],
                            &output[..$arr.len() - 1],
                            "Rx len < Tx len, bit order {}",
                            bit_order_name
                        );
                        input.fill(0);

                        spi.transfer(&mut input, &output[..$arr.len() - 1]).unwrap();
                        let mut expected = $arr;
                        *expected.last_mut().unwrap() = u32::MAX as _;
                        defmt::assert_eq!(
                            input,
                            expected,
                            "Rx > Tx len, bit order {}",
                            bit_order_name
                        );
                        input.fill(0);

                        spi.transfer(&mut input[2..], &output).unwrap();
                        defmt::assert_eq!(
                            &input[2..],
                            &output[..$arr.len() - 2],
                            "Rx len << Tx len, bit order {}",
                            bit_order_name
                        );
                        input.fill(0);

                        spi.transfer(&mut input, &output[..$arr.len() - 2]).unwrap();
                        let mut expected = $arr;
                        expected[$arr.len() - 1] = u32::MAX as _;
                        expected[$arr.len() - 2] = u32::MAX as _;
                        defmt::assert_eq!(
                            input,
                            expected,
                            "Rx >> Tx len, bit order {}",
                            bit_order_name
                        );
                        input.fill(0);
                    };
                }

                transfer_test!([1u8, 2, 3], BitOrder::Msb);
                transfer_test!([1u8, 2, 3], BitOrder::Lsb);

                transfer_test!([1u8, 2, 3, 4], BitOrder::Msb);
                transfer_test!([1u8, 2, 3, 4], BitOrder::Lsb);

                transfer_test!([1u8, 2, 3, 4, 5], BitOrder::Msb);
                transfer_test!([1u8, 2, 3, 4, 5], BitOrder::Lsb);

                transfer_test!([1u8, 2, 3, 4, 5, 6], BitOrder::Msb);
                transfer_test!([1u8, 2, 3, 4, 5, 6], BitOrder::Lsb);

                transfer_test!([1u8, 2, 3, 4, 5, 6, 7], BitOrder::Msb);
                transfer_test!([1u8, 2, 3, 4, 5, 6, 7], BitOrder::Lsb);

                transfer_test!([0x0102u16, 0x0304, 0x0506], BitOrder::Msb);
                transfer_test!([0x0102u16, 0x0304, 0x0506], BitOrder::Lsb);

                transfer_test!([0x0102u16, 0x0304, 0x0506, 0x0708], BitOrder::Msb);
                transfer_test!([0x0102u16, 0x0304, 0x0506, 0x0708], BitOrder::Lsb);

                transfer_test!([0x0102u16, 0x0304, 0x0506, 0x0708, 0x090A], BitOrder::Msb);
                transfer_test!([0x0102u16, 0x0304, 0x0506, 0x0708, 0x090A], BitOrder::Lsb);

                transfer_test!([0x01020304u32, 0x05060708, 0x090A0B0C], BitOrder::Msb);
                transfer_test!([0x01020304u32, 0x05060708, 0x090A0B0C], BitOrder::Lsb);

                spi.set_bit_order(BitOrder::Msb);
                delay();
            }

            {
                // Change me to test different Elem sizes, buffer sizes,
                // bit patterns.
                type Elem = u8;
                const SENTINEL: Elem = 0x0F;
                const BUFFER: [Elem; 13] = [SENTINEL; 13];

                // Simple loopback transfer. Easy to find with your
                // scope.
                let mut buffer = BUFFER;
                spi.transfer_in_place(&mut buffer).unwrap();
                if buffer != BUFFER {
                    defmt::error!("Simple transfer buffer mismatch!");
                }

                delay();

                // Adjacent loopback transfer. Look for the big
                // burst of data on your scope.
                let mut buffer = BUFFER;
                let mut error = false;
                for idx in 0u32..16 {
                    buffer.fill(SENTINEL.rotate_right(idx));
                    let expected = buffer;
                    spi.transfer_in_place(&mut buffer).unwrap();
                    error |= buffer != expected;
                }
                if error {
                    defmt::error!("At least one of the bursted transfers didn't match!");
                }

                delay();

                // Simple write.
                let buffer = BUFFER;
                spi.write(&buffer).unwrap();

                delay();

                // Pipelined writes. Look for the burst of data
                // on your scope. The embedded-hal 1.0 writes do
                // not flush, so the delay between subsequent
                // write operations should be smaller than bi-
                // directional transfers
                let mut buffer = BUFFER;
                for idx in 0..16 {
                    buffer.fill(SENTINEL.rotate_right(idx));
                    spi.write(&buffer).unwrap();
                }

                delay();
            }
        }
    }
}
