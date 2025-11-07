//! Demonstrates imxrt-usbh, a USB host driver.

#![no_std]
#![no_main]

#[rtic::app(device = board, peripherals = false, dispatchers = [BOARD_SWTASK0])]
mod app {

    use imxrt_hal as hal;

    use rtic_monotonics::systick::{prelude::*, SystClkSource};
    systick_monotonic!(Mono);

    #[local]
    struct Local {
        /// An LED for fun!
        led: board::Led,
        /// The USB host controller.
        usb_host: hal::usbh::HostController<2>,
        /// If enabled, drives USB logging.
        poller: Option<board::logging::Poller>,
    }

    #[shared]
    struct Shared {}

    #[init]
    fn init(cx: init::Context) -> (Shared, Local) {
        let (
            board::Common {
                usb1,
                usbnc1,
                usbphy1,
                ..
            },
            board::Specifics { led, usb_host, .. },
        ) = board::new();

        Mono::start_with_clock_source(cx.core.SYST, 100_000, SystClkSource::External);

        let usbd = hal::usbd::Instances {
            usb: usb1,
            usbnc: usbnc1,
            usbphy: usbphy1,
        };

        let poller = board::defmt_transport::init(usbd);

        usb_host_task::spawn().unwrap();

        (
            Shared {},
            Local {
                led,
                poller,
                usb_host,
            },
        )
    }

    #[task(local = [led, usb_host, counter: u32 = 0])]
    async fn usb_host_task(cx: usb_host_task::Context) {
        let usb_host_task::LocalResources { led, counter, .. } = cx.local;
        loop {
            led.toggle();
            defmt::println!("Hello world! {=u32}", counter);
            Mono::delay(250_u32.millis()).await;
            *counter = counter.wrapping_add(1);
        }
    }

    /// Pump defmt logs over USB if required by the board.
    #[task(binds = BOARD_USB1, local = [poller])]
    fn usb_interrupt(cx: usb_interrupt::Context) {
        cx.local.poller.as_mut().unwrap().poll();
    }
}
