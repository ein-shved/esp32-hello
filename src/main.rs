#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]

use embassy_executor::Spawner;
use esp_hal::clock::CpuClock;
use esp_hal::gpio::{Io, Level, Output, OutputConfig};
use esp_hal::peripherals::{IEEE802154, Peripherals};
use esp_hal::time::{Duration, Instant};
use esp_radio::ieee802154::Ieee802154;
use log::info;
use zigbee::nwk::nlme::Nlme;
use zigbee::nwk::nlme::management::NetworkDescriptor;
use zigbee_mac::esp::EspMlme;
use {esp_backtrace as _, esp_println as _};

// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();

const SCAN_DURATION: u8 = 10u8;
const CHANNELS: core::ops::Range<u8> = 11..27;

#[esp_rtos::main]
async fn main(spawner: embassy_executor::Spawner) {
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(esp_hal::Config::default());
    use esp_hal::timer::timg::TimerGroup;
    let timg0 = TimerGroup::new(peripherals.TIMG0);

    esp_alloc::heap_allocator!(size: 24 * 1024);

    use esp_hal::interrupt::software::SoftwareInterruptControl;
    let software_interrupt = SoftwareInterruptControl::new(peripherals.SW_INTERRUPT);

    esp_rtos::start(timg0.timer0, software_interrupt.software_interrupt0);

    let mut led = Output::new(peripherals.GPIO15, Level::High, OutputConfig::default());

    esp_println::logger::init_logger(log::LevelFilter::Info);

    spawner.spawn(discovery(peripherals.IEEE802154));

    loop {
        info!("Hello world!");
        led.toggle();
        let delay_start = Instant::now();
        embassy_time::Timer::after_millis(500).await;
    }

    // for inspiration have a look at the examples at https://github.com/esp-rs/esp-hal/tree/esp-hal-v1.0.0/examples/src/bin
}

#[embassy_executor::task]
async fn discovery(ieee802154: IEEE802154<'static>) {
    zigbee::nwk::nib::init(zigbee::nwk::nib::NibStorage::default());
    let ieee802154 = Ieee802154::new(ieee802154);
    let mac = EspMlme::new(ieee802154, Default::default());

    let mut nwk = Nlme::new(mac);

    loop {
        info!("Discovery started!");
        match nwk.network_discovery(CHANNELS, SCAN_DURATION).await {
            Ok(nd) => for nd in nd.network_descriptor.into_iter() {
                info!("Network descriptor: {nd:?}");
            }
            Err(err) => info!("Discovery failed: {err}"),
        }
        let delay_start = Instant::now();
        embassy_time::Timer::after_secs(2).await;
    }
}
