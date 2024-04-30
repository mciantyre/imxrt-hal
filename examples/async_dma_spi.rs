//! Demonstrates full-duplex, async SPI I/O.
//!
//! Connect SDI to SDO to enable loopback transfers. If there's no loopback,
//! the example panics. Use this to interrupt an otherwise working example.

#![no_std]
#![no_main]

#[imxrt_rt::entry]
fn main() -> ! {
    let (
        board::Common {
            mut dma,
            pit: (mut pit, _, _, _),
            ..
        },
        board::Specifics { mut spi, .. },
    ) = board::new();

    let mut chan_a = dma[board::BOARD_DMA_A_INDEX].take().unwrap();
    chan_a.set_disable_on_completion(true);

    pit.set_load_timer_value(board::PIT_FREQUENCY / 1000 * 250);

    let mut delay = move || {
        pit.enable();
        while !pit.is_elapsed() {}
        pit.clear_elapsed();
        pit.disable();
    };

    let task = async {
        let mut trans = imxrt_hal::lpspi::Transaction::new(16).unwrap();
        trans.receive_data_mask = true;
        spi.enqueue_transaction(&trans);

        let mut elem: u32 = 0x01020304;

        loop {
            let outgoing = [elem; 96];
            imxrt_hal::dma::peripheral::write(&mut chan_a, &outgoing, &mut spi)
                .await
                .unwrap();
            delay();
        }
    };
    pin_utils::pin_mut!(task);
    board::blocking::run(task);
    unreachable!();
}
