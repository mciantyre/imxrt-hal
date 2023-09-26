#![no_std]

use core::pin::Pin;
use imxrt_ral as ral;

#[cxx::bridge(namespace = "imxrt")]
mod ffi {
    unsafe extern "C++" {
        include!("cxx-pit/include/pit.hpp");

        type PitChannel;

        fn set_load_timer_value(&self, ticks: u32);
        fn enable(self: Pin<&mut PitChannel>);
        fn is_elapsed(&self) -> bool;
        fn clear_elapsed(&self);

        unsafe fn initialize_pit() -> [*mut PitChannel; 4];
    }
}

pub use ffi::PitChannel;
pub type Channels = [Pin<&'static mut PitChannel>; 4];

/// Initialize the PIT, and return handles to the four timer channels.
///
/// Use this as the entry point to safely acquire PIT channels.
pub fn new(_: ral::pit::PIT) -> Channels {
    // Safety: we take ownership of the PIT peripheral instance. The caller already
    // ensures that they have the only valid instance in the program.
    let [a, b, c, d] = unsafe { ffi::initialize_pit() };

    // Safety: by construction of the C++ library, the objects have static lifetime.
    // Since the caller ensures that they have the only safe instance of the PIT
    // peripheral instance, there's no chance for multiple mutable references.
    unsafe {
        [
            Pin::new_unchecked(&mut *a),
            Pin::new_unchecked(&mut *b),
            Pin::new_unchecked(&mut *c),
            Pin::new_unchecked(&mut *d),
        ]
    }
}
