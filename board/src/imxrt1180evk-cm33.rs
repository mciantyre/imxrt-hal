//! i.MX RT 1180 EVK, supporting the Cortex-M33.

use imxrt_hal::{self as hal, iomuxc};
use imxrt_iomuxc::imxrt1180::gpio_ad::*;
use imxrt_ral as ral;

#[cfg(target_arch = "arm")]
use defmt_rtt as _;
#[cfg(target_arch = "arm")]
use imxrt1180evk_fcb as _;

use panic_probe as _;

pub unsafe fn configure() {}

/// Runs on the OSC_RC_24M by default. Lucky guess!
pub const UART_CLK_FREQUENCY: u32 = 24_000_000;
/// TODO: I'm making this up. Don't make it up.
pub const LPI2C_CLK_FREQUENCY: u32 = 24_000_000;

/// USER_LED1 on the board.
///
/// Managed through GPIO4_27.
pub type Led = imxrt_hal::rgpio::Output<GPIO_AD_27>;

const CONSOLE_INSTANCE: u8 = 1;
pub type Console = hal::lpuart::Lpuart<(), { CONSOLE_INSTANCE }>;

pub const CONSOLE_BAUD: hal::lpuart::Baud = hal::lpuart::Baud::compute(UART_CLK_FREQUENCY, 115200);

#[non_exhaustive]
pub struct Specifics {
    pub led: Led,
    pub console: Console,
}

impl Specifics {
    pub(crate) fn new(_: &mut crate::Common) -> Self {
        let ral::Instances {
            IOMUXC,
            IOMUXC_AON,
            RGPIO4,
            LPUART1,
            ..
        } = unsafe { ral::Instances::instances() };
        let mut pads = imxrt_hal::iomuxc::into_pads(IOMUXC, IOMUXC_AON);

        iomuxc::alternate(&mut pads.gpio_aon.p08, 0); // LPUART1_TX
        iomuxc::alternate(&mut pads.gpio_aon.p09, 0); // LPUART1_RX

        let mut gpio4 = imxrt_hal::rgpio::Port::new(RGPIO4);
        let led = gpio4.output(pads.gpio_ad.p27);

        let mut console = hal::lpuart::Lpuart::without_pins(LPUART1);
        console.disable(|console| {
            console.set_baud(&CONSOLE_BAUD);
            console.set_parity(None);
        });

        Specifics { led, console }
    }
}

pub mod interrupt {
    use crate::board_interrupts as syms;
    use crate::ral::Interrupt;

    pub const INTERRUPTS: &[(Interrupt, syms::Vector)] = &[];
}
