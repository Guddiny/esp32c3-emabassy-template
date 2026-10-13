use crate::shared::LED_CHANNEL;
use defmt::info;
use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use esp_hal::{
    gpio::{Level, Output, OutputConfig},
    peripherals::GPIO8,
};

pub fn start_led_service(spawner: Spawner, peripherals_pin: GPIO8<'static>) {
    let led = Output::new(peripherals_pin, Level::High, OutputConfig::default());

    spawner.spawn(led_task(led).unwrap());
}

#[embassy_executor::task]
async fn led_task(mut led: Output<'static>) {
    let receiver = LED_CHANNEL.receiver();

    loop {
        let signal = receiver.receive().await;
        info!("Message received");
        for _ in 0..signal {
            led.set_low();
            Timer::after(Duration::from_millis(50)).await;
            led.set_high();
            Timer::after(Duration::from_millis(70)).await;
        }
    }
}
