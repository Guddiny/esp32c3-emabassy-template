use defmt::info;
use embassy_executor::Spawner;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;
use embassy_time::{Duration, Timer};
use esp_hal::gpio::{Level, Output, OutputConfig};

use crate::board::board::LedPin;

pub static LED_CHANNEL: Channel<CriticalSectionRawMutex, LedSignal, 4> = Channel::new();

#[derive(Clone, Copy, Debug)]
pub enum LedSignal {
    OK,
    Error,
}

impl LedSignal {
    pub fn blink_count(self) -> u8 {
        match self {
            Self::OK => 1,
            Self::Error => 4,
        }
    }
}

pub fn start_led_driver(spawner: Spawner, led_pin: LedPin) {
    let led = Output::new(led_pin.0, Level::High, OutputConfig::default());

    spawner.spawn(led_task(led).unwrap());
}

#[embassy_executor::task]
async fn led_task(mut led: Output<'static>) {
    let receiver = LED_CHANNEL.receiver();

    loop {
        let signal = receiver.receive().await;
        //info!("Message received");
        for _ in 0..signal.blink_count() {
            led.set_low();
            Timer::after(Duration::from_millis(50)).await;
            led.set_high();
            Timer::after(Duration::from_millis(70)).await;
        }
    }
}
