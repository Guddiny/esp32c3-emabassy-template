use defmt::{error, info};
use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use esp_hal::gpio::{Input, InputConfig};
use heapless::String;

use crate::board::board::ButtonPin;
use crate::services::notifications::{Log, LogLevel, Notification, OutboundPackage};
use crate::services::led::{LED_CHANNEL, LedSignal};

pub fn start_button_driver(spawner: Spawner, button_pin: ButtonPin) {
    let button = Input::new(button_pin.0, InputConfig::default());

    spawner.spawn(button_service(button).unwrap());
}

#[embassy_executor::task]
async fn button_service(mut button: Input<'static>) {
    loop {
        button.wait_for_falling_edge().await;

        Timer::after(Duration::from_millis(30)).await;

        if button.is_low() {
            info!("Buttons pressed");
            let log = Log {
                level: LogLevel::Info,
                message: String::try_from("New log message ").unwrap(),
            };
            let pkg = OutboundPackage::Log(&log);
            Notification::publish(&pkg).await;
            match LED_CHANNEL.try_send(LedSignal::OK) {
                Ok(_) => info!("LED message sent"),
                Err(e) => error!("{:?}", defmt::Debug2Format(&e)),
            }
        }
    }
}
