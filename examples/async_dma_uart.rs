//! Tests for the LPUART async DMA interface.
//!
//! Wait for a serial character, then send the same
//! character back 32 times. The LED toggles after
//! every round trip.

#![no_std]
#![no_main]

#[imxrt_rt::entry]
fn main() -> ! {
    let (
        board::Common { mut dma, .. },
        board::Specifics {
            led, mut console, ..
        },
    ) = board::new();

    let mut channel = dma[board::BOARD_DMA_A_INDEX].take().unwrap();
    channel.set_disable_on_completion(true);

    let task = async {
        loop {
            led.toggle();

            let mut receive = [0u8; 2];
            console.dma_read(&mut channel, &mut receive).await.unwrap();

            let mut transmit = [0; 32];
            for pairs in transmit.chunks_exact_mut(2) {
                pairs[0] = receive[0];
                pairs[1] = receive[1];
            }
            console.dma_write(&mut channel, &transmit).await.unwrap();
        }
    };
    pin_utils::pin_mut!(task);
    board::blocking::run(task);
    unreachable!();
}
