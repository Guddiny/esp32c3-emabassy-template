#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]
#![deny(clippy::large_stack_frames)]

use defmt::{error, info};
use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use esp_hal::clock::CpuClock;
use esp_hal::timer::timg::TimerGroup;
use esp_println as _;
use my_esp::{dhcp_server, led_service, rf_sensor, web_server, wifi, button_service};

#[panic_handler]
fn panic(panic_info: &core::panic::PanicInfo) -> ! {
    error!("{}", panic_info);
    loop {}
}

extern crate alloc;

// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();

#[allow(
    clippy::large_stack_frames,
    reason = "it's not unusual to allocate larger buffers etc. in main"
)]
#[esp_rtos::main]
async fn main(spawner: Spawner) -> ! {
    // generator version: 1.4.0
    // generator parameters: -o esp32c3 -o esp32c3-mini-1 -o alloc -o unstable-hal -o embassy -o wifi -o defmt -o agents -o zed -o stable-aarch64-apple-darwin

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    esp_alloc::heap_allocator!(#[esp_hal::ram(reclaimed)] size: 66320);

    let timg0 = TimerGroup::new(peripherals.TIMG0);
    esp_rtos::start(timg0.timer0, peripherals.FROM_CPU_INTR0);

    info!("Embassy initialized!");

    let stack = wifi::start_wifi(spawner, peripherals.WIFI).await;
    web_server::start_web_server(spawner, stack).await;
    dhcp_server::start_dhcp_server(spawner, stack).await;
    rf_sensor::start_rf_sensor(
        spawner,
        peripherals.SPI2,
        peripherals.GPIO4,
        peripherals.GPIO10,
        peripherals.GPIO12,
    );
    led_service::start_led_service(spawner, peripherals.GPIO8);
    button_service::start_button_service(spawner, peripherals.GPIO9);

    loop {
        info!("Main");
        Timer::after(Duration::from_secs(60)).await;
    }
}
