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
use heapless::String;
use my_esp::app::state::{AppState, Settings, State};
use my_esp::board::board::Board;
use my_esp::services::notifications::{Messaging, OutboundPackage};
use my_esp::services::ui::funcui::{ACTION_CHANNEL, Alignment, Builder, InputType, REDRAW_UI};
use my_esp::services::*;

extern crate alloc;

#[panic_handler]
fn panic(panic_info: &core::panic::PanicInfo) -> ! {
    error!("{}", panic_info);
    loop {}
}

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
    let board = Board::init(peripherals);

    esp_alloc::heap_allocator!(#[esp_hal::ram(reclaimed)] size: 66320);

    let timg0 = TimerGroup::new(board.rtos_timer);
    esp_rtos::start(timg0.timer0, board.rtos_irq);

    info!("Embassy initialized!");

    let stack = inet::wifi::start_wifi(spawner, board.wifi).await;
    inet::web_server::start_web_server(spawner, stack).await;
    inet::dhcp_server::start_dhcp_server(spawner, stack).await;
    radio::start_rf_sensor(spawner, board.radio);
    led::start_led_driver(spawner, board.led);
    button::start_button_driver(spawner, board.button);

    let mut app_state = AppState {
        settings: Settings {
            led_enabled: true,
            wifi_ssid: String::try_from("12345678").unwrap_or_default(),

            min_vol: 1,
            max_vol: 12,
            current_vol: 4,
        },
        state: State { is_running: false },
    };

    loop {
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
        build(&mut builder, &mut app_state);

        let pkg = OutboundPackage::Data(builder.packet.as_slice());

        Messaging::publish(&pkg).await;
    }
}

fn build(b: &mut Builder, app_state: &mut AppState) {
    b.tab("Main", false, |b| {
        b.group("", false, Alignment::Vertical, |b| {
            b.group("wifi", false, Alignment::Vertical, |b| {
                b.input(
                    "SSID",
                    "SSID",
                    InputType::Text,
                    &mut app_state.settings.wifi_ssid,
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
                b.toggle("Echo Mode", &mut app_state.settings.led_enabled)
                    .notify_on_change();
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
                app_state.settings.min_vol,
                app_state.settings.max_vol,
                &mut app_state.settings.current_vol,
            );
            b.slider(
                "Brightness",
                app_state.settings.min_vol,
                app_state.settings.max_vol,
                &mut app_state.settings.current_vol,
            );
            b.select("Items", &["OPtion 1", "Option 2", "option 3"], &mut 1);
            b.progress_bar("Progress bar 1", "Some text", &mut 0);
            b.progress_bar("Progress bar 2", "Some text", &mut 50);
            b.progress_ring("Progress bar 2", &mut true);
        });
    })
    .tab("Logs", false, |b| {
        b.group("", false, Alignment::Vertical, |b| {
            b.log("", "");
        });
    });
}
