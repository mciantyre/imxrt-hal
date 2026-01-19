//! The chip API when no chip is selected.

pub use drivers::flexio;

pub mod ccm {
    pub use crate::common::ccm::*;
}
pub mod dma {}

pub(crate) mod iomuxc {}

mod drivers {
    pub mod flexio;
}

#[path = "drivers"]
pub(crate) mod config {
    #[path = "flexio"]
    pub(crate) mod flexio {
        mod shifter4;
        mod timer4;

        pub use shifter4::*;
        pub use timer4::*;
    }
}
