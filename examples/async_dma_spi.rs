//! Demonstrates full-duplex, async SPI I/O.
//!
//! Connect SDI to SDO to enable loopback transfers. If there's no loopback,
//! the example panics. Use this to interrupt an otherwise working example.

#![no_std]
#![no_main]

use hal::dma::channel::Channel;
use imxrt_hal as hal;

/// We want the DMA engine to perform 16 bit reads from the buffer and 16 bit writes
/// to the LPSPI transmit data register. This manually sets up that transaction.
///
/// We need to write this manually because today's LPSPI implementation only supports
/// 32-bit DMA within a single transaction.
async fn do_custom_dma(channel: &mut Channel, source: &[u16], destination: &mut board::Spi) {
    use hal::dma::{
        channel::{self, Configuration},
        peripheral::Destination,
    };

    // These persist across calls, so they could be committed once on the channel
    // instead on every invocation of this function.
    channel.set_channel_configuration(Configuration::enable(destination.destination_signal()));
    unsafe {
        // Treat the buffer as a collection of u16s. This tells the DMA engine
        // to perform 16 bit reads.
        channel::set_source_linear_buffer(channel, source);

        // Treat the hardware port as a u16. This tells the DMA engine to perform
        // 16 bit writes.
        let destination_address: *const u32 = destination.destination_address();
        let destination_address: *const u16 = destination_address.cast();
        channel::set_destination_hardware(channel, destination_address);

        // Move 2 bytes (16 bits) on every service request. By the above configurations,
        // the DMA engine understands it should perform one read, then one write.
        channel.set_minor_loop_bytes(core::mem::size_of::<u16>() as u32);
        // Move every element in the buffer.
        channel.set_transfer_iterations(source.len() as u16);

        destination.enable_destination();

        let xfer = hal::dma::Transfer::new(channel);
        xfer.await.unwrap();

        destination.disable_destination();
        while channel.is_hardware_signaling() {}
    }
}

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

        loop {
            let mut outgoing = [0u16; 256];
            for (idx, elem) in outgoing.iter_mut().enumerate() {
                *elem = idx as u16;
            }
            do_custom_dma(&mut chan_a, &outgoing, &mut spi).await;
            delay();
        }
    };
    pin_utils::pin_mut!(task);
    board::blocking::run(task);
    unreachable!();
}
