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
use embassy_futures::select::*;
use esp_hal::clock::CpuClock;
use esp_hal::timer::timg::TimerGroup;
use esp_println as _;
use heapless::{String, Vec};
use my_esp::{
    button_service, dhcp_server, led_service, rf_sensor,
    shared::{ACTION_CHANNEL, PACKET_CHANNEL},
    state::Settings,
    ui::funcui::{Alignment, Builder, InputType, REDRAW_UI},
    web_server, wifi,
};

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
        peripherals.GPIO10,
        peripherals.GPIO4,
        peripherals.GPIO0,
    );
    led_service::start_led_service(spawner, peripherals.GPIO8);
    button_service::start_button_service(spawner, peripherals.GPIO9);

    let mut settings = Settings {
        led_enabled: true,
        wifi_ssid: String::try_from("12345678").unwrap_or_default(),

        min_vol: 1,
        max_vol: 12,
        current_vol: 4,
    };

    loop {
        let sender = PACKET_CHANNEL.sender();

        let event = select(ACTION_CHANNEL.receive(), REDRAW_UI.wait()).await;

        let action = match event {
            Either::First(e) => Some(e),
            Either::Second(_) => None,
        };

        if action.is_some() {
            info!("ACTION RECEIVED: {}", defmt::Debug2Format(&action));
        }

        if action.is_none() {
            info!(
                "UI_REBUILD_TRIGGER RECEIVED: {}",
                defmt::Debug2Format(&action)
            );
        }

        let mut builder = Builder::new(action);
        build(&mut builder, &mut settings);

        let mut json = heapless::String::<{ my_esp::config::MAX_PACKET_SIZE }>::new();
        if serde_json_core::to_string::<_, { my_esp::config::MAX_PACKET_SIZE }>(&builder.packet)
            .ok()
            .map(|s| {
                let _ = json.push_str(&s);
            })
            .is_none()
        {
            error!("JSON serialize failed");
            continue;
        }

        info!("{}", &json);

        let mut vec = heapless::Vec::<u8, { my_esp::config::MAX_PACKET_SIZE }>::new();
        if vec.extend_from_slice(json.as_bytes()).is_err() {
            error!("Packet too large");
            continue;
        }

        info!("Publishing packet: {} bytes", vec.len());
        let _ = sender.send(vec).await;
    }
}

fn build(b: &mut Builder, settings: &mut Settings) {
    b.tab("Main", false, |b| {
        b.group("", false, Alignment::Vertical, |b| {
            b.group("wifi", false, Alignment::Vertical, |b| {
                b.input(
                    "SSID",
                    "SSID",
                    InputType::Text,
                    &mut String::try_from("K92_new").unwrap(),
                );
                b.input(
                    "Password",
                    "Password",
                    InputType::Password,
                    &mut String::try_from("123").unwrap(),
                );

                b.button("Connect", "Connect Btn");
            });

            b.group("User settings", false, Alignment::Vertical, |b| {
                b.toggle("Echo Mode", &mut true).notify_on_change();
                b.button("Connect", "Connect Btn");

                b.group("Email", false, Alignment::Horizontal, |b| {
                    b.input(
                        "Email",
                        "Email",
                        InputType::Email,
                        &mut String::try_from("my@email.com").unwrap(),
                    );
                    b.button("Add", "Add Email");
                });
            });
        });
    })
    .tab("Settings", false, |b| {
        b.group("", false, Alignment::Vertical, |b| {
            b.slider(
                "Volume2",
                settings.min_vol,
                settings.max_vol,
                &mut settings.current_vol,
            );
            b.slider(
                "Brightness",
                settings.min_vol,
                settings.max_vol,
                &mut settings.current_vol,
            );
            b.select("Items", &["OPtion 1", "Option 2", "option 3"], &mut 1);
            b.progress_bar("Progress bar 1", "Some text", &mut 0);
            b.progress_bar("Progress bar 2", "Some text", &mut 50);
            b.progress_ring("Progress bar 2", &mut true);
        });
    })
    .tab("Logs", false, |b| {
        b.group("", false, Alignment::Vertical, |b| {
            b.group("", false, Alignment::Horizontal, |b| {
                b.label("", String::try_from("Logs").as_mut().unwrap());
                b.button("Clear", "Clear btn");
            });
        });
    });
}
