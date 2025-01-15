pub use drivers::{ccm_118x as ccm, dma, rgpio};

pub(crate) mod iomuxc {
    pub use super::config::pads;
    use crate::ral;

    /// Transform the `imxrt-ral` IOMUXC instances into pad objects.
    pub fn into_pads(_: ral::iomuxc::IOMUXC, _: ral::iomuxc_aon::IOMUXC_AON) -> pads::Pads {
        // Safety: acquiring pads has the same safety implications
        // as acquiring the IOMUXC instances. The user has already
        // assumed the unsafety.
        unsafe { pads::Pads::new() }
    }
}

mod drivers {
    pub mod ccm_118x;
    pub mod dma;
    pub mod rgpio;
}

pub(crate) mod config {
    pub use imxrt_iomuxc::imxrt1180 as pads;

    /// The minimum number of DMA channels per controller.
    pub const DMA_CHANNEL_COUNT: usize = 32;
}
