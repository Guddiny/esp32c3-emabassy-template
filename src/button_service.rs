use defmt::{error, info};
use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use esp_hal::{
    gpio::{Input, InputConfig},
    peripherals::GPIO9,
};

use crate::shared::{LED_CHANNEL, LedSignal};

pub fn start_button_service(spawner: Spawner, button_pin: GPIO9<'static>) {
    let button = Input::new(button_pin, InputConfig::default());

    spawner.spawn(button_service(button).unwrap());
}

#[embassy_executor::task]
async fn button_service(mut button: Input<'static>) {
    let led_sender = LED_CHANNEL.sender();

    loop {
        button.wait_for_falling_edge().await;
        Timer::after(Duration::from_millis(30)).await;

        if button.is_low() {
            info!("Buttons pressed");
            match led_sender.try_send(LedSignal::OK) {
                Ok(_) => info!("LED message sent"),
                Err(e) => error!("{:?}", defmt::Debug2Format(&e)),
            }
        }
    }
}
