//! Demonstrates full-duplex, async SPI I/O.
//!
//! Connect SDI to SDO to enable loopback transfers. If there's no loopback,
//! the example panics. Use this to interrupt an otherwise working example.

#![no_std]
#![no_main]

extern crate alloc;
use alloc::boxed::Box;

use core::{
    future::Future,
    task::{Context, Waker},
};

use linked_list_allocator::LockedHeap;

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

fn init_heap() {
    let heap_start = imxrt_rt::heap_start();
    let heap_end = imxrt_rt::heap_end() as usize;
    let heap_size = heap_end - heap_start as usize;
    unsafe {
        ALLOCATOR.lock().init(heap_start.cast(), heap_size);
    }
}

fn nop_waker() -> Waker {
    use core::task::{RawWaker, RawWakerVTable};
    const VTABLE: RawWakerVTable = RawWakerVTable::new(|_| RAW_WAKER, |_| {}, |_| {}, |_| {});

    const RAW_WAKER: RawWaker = RawWaker::new(core::ptr::null(), &VTABLE);
    // Safety: raw waker meets documented requirements.
    unsafe { Waker::from_raw(RAW_WAKER) }
}

#[imxrt_rt::entry]
fn main() -> ! {
    init_heap();
    let (board::Common { mut dma, .. }, board::Specifics { mut spi, .. }) = board::new();

    let mut chan_a = dma[board::BOARD_DMA_A_INDEX].take().unwrap();
    chan_a.set_disable_on_completion(true);

    let buffer = &mut [0u32; 32];
    // dma_obj borrows stack-allocated buffer.
    let dma_obj = spi.dma_read(&mut chan_a, buffer).unwrap();
    // dma_obj moved and pinned to heap.
    let mut dma_obj = Box::pin(dma_obj);

    // Start the DMA transfer into stack memory.
    let _ = dma_obj
        .as_mut()
        .poll(&mut Context::from_waker(&nop_waker()));

    // Lose access to the DMA transfer, preventing drop.
    // This statisfies the Pin drop guarantee while corrupting
    // stack memory.
    //
    // Initial design always assumes stack-allocated DMA objects,
    // which we can't forget without violating the drop guarantee.
    // That assumption doesn't hold for heap pinning.
    core::mem::forget(dma_obj);

    unreachable!();
}
