//! Demonstrates uSDHC commands.
//!
//! Slide an SD card into your board. This example uses low-level SDIO commands
//! to interface your card. If everything goes well, the LED turns on and the example
//! ends there. Check your logging output for specific device information.

#![no_main]
#![no_std]

use imxrt_hal as hal;

/// Change me to change how log messages are serialized
/// and transported.
const FRONTEND: board::logging::Frontend = board::logging::Frontend::Log;
/// This is a function of your board. Want to change it? Change it right
/// here to explore different example code paths.
const BACKEND: board::logging::Backend = board::logging::BACKEND;

#[imxrt_rt::entry]
fn main() -> ! {
    let (
        board::Common {
            pit: (pit, _, _, _),
            usb1,
            usbnc1,
            usbphy1,
            mut dma,
            usdhc,
            ..
        },
        board::Specifics { led, console, .. },
    ) = board::new();

    let mut delay = hal::timer::Blocking::<_, { board::PIT_FREQUENCY }>::from_pit(pit);

    let usbd = hal::usbd::Instances {
        usb: usb1,
        usbnc: usbnc1,
        usbphy: usbphy1,
    };

    let dma_a = dma[board::BOARD_DMA_A_INDEX].take().unwrap();
    let mut poller = board::logging::init(FRONTEND, BACKEND, console, dma_a, usbd);

    delay.block_ms(1000);

    let host = match hal::usdhc::BlockingSdioHost::new(usdhc, &mut |ms| delay.block_ms(ms)) {
        Ok(host) => host,
        Err(err) => {
            log::error!("{:?}", err);
            poller.poll();
            panic!("{:?}", err);
        }
    };

    log::info!("RCA: {:#06X}", host.rca().address());
    poller.poll();
    log::info!("{:?}", host.cid());
    poller.poll();
    log::info!("{:?}", host.csd());
    poller.poll();
    log::info!("{:?}", host.scr());
    poller.poll();
    log::info!("{:?}", host.sd_status());
    poller.poll();

    delay.block_ms(100);

    {
        let mut ctrl = Controller::new(host.into_sdmmc_block_dev(), NopTimeSource);
        poller.poll();

        let mut vol = ctrl.get_volume(VolumeIdx(0)).unwrap();
        let dir = ctrl.open_root_dir(&vol).unwrap();

        let mut secret = ctrl
            .open_file_in_dir(
                &mut vol,
                &dir,
                "SECRET.txt",
                Mode::ReadWriteCreateOrTruncate,
            )
            .unwrap();

        ctrl.write(&mut vol, &mut secret, b"You found the secret code!")
            .unwrap();
        poller.poll();

        ctrl.close_file(&vol, secret).unwrap();
        ctrl.close_dir(&vol, dir);
    }

    loop {
        led.set();
        poller.poll();
    }
}

use board::sdmmc::{
    filesystem::{Mode, TimeSource, Timestamp},
    Controller, VolumeIdx,
};

struct NopTimeSource;

impl TimeSource for NopTimeSource {
    fn get_timestamp(&self) -> Timestamp {
        Timestamp {
            year_since_1970: 0,
            zero_indexed_month: 0,
            zero_indexed_day: 0,
            hours: 0,
            minutes: 0,
            seconds: 0,
        }
    }
}
