//! Demonstrates full-duplex, async SPI I/O.
//!
//! Connect SDI to SDO to enable loopback transfers. If there's no loopback,
//! the example panics. Use this to interrupt an otherwise working example.

#![no_std]
#![no_main]

#[imxrt_rt::entry]
fn main() -> ! {
    let (board::Common { mut dma, .. }, board::Specifics { mut spi, .. }) = board::new();

    let mut chan_a = dma[board::BOARD_DMA_A_INDEX].take().unwrap();
    chan_a.set_disable_on_completion(true);

    let data: [u32; 5] = [1, 2, 3, 4, 5];
    {
        let dma_transfer = spi.dma_write(&mut chan_a, &data).unwrap();
        cassette::pin_mut!(dma_transfer);
        is_pinned_write_object(&dma_transfer); // <-- NEW: demonstrate shadowing and pinning.
        let mut cm = cassette::Cassette::new(dma_transfer);
        cm.poll_on();
        core::mem::forget(cm);
    }

    drop(data);
    unreachable!();
}

use core::pin::Pin;

/// Shorthand for the `Write` object...
type Write<'a> = imxrt_hal::dma::peripheral::Write<'a, board::Spi, u32>;

fn is_pinned_write_object(_: &Pin<&mut Write<'_>>) {}
