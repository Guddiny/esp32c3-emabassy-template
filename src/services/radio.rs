use cc1101_embassy::{Cc1101, Modulation, RadioConfig, TxPower};
use defmt::{error, info};
use embassy_embedded_hal::shared_bus::asynch::spi::SpiDevice;
use embassy_executor::Spawner;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::mutex::Mutex;
use embassy_time::{Duration, Timer};
use esp_hal::Async;
use esp_hal::gpio::{Input, InputConfig, Level, Output, OutputConfig};
use esp_hal::spi::master::{Config, Spi};
use heapless::String;
use static_cell::StaticCell;

use crate::app::state::Settings;
use crate::board::board::RadioSpiPins;
use crate::services::notifications::{MessageKind, Messaging, OutboundPackage};

static RADIO_SPI_BUS: StaticCell<Mutex<CriticalSectionRawMutex, Spi<'static, Async>>> =
    StaticCell::new();
static SETTINGS: StaticCell<Settings> = StaticCell::new();

pub fn start_rf_sensor(spawner: Spawner, radio: RadioSpiPins) {
    // #define SS_CS_PIN 10
    // #define SCK_PIN 6
    // #define MOSI_PIN 7
    // #define MISO_PIN 2
    // #define GD0 4
    // #define GD2 0

    let spi = Spi::new(radio.spi.0, Config::default())
        .unwrap()
        .into_async();
    let spi_bus = RADIO_SPI_BUS.init(Mutex::new(spi));
    let cs_pin = Output::new(radio.cs.0, Level::High, OutputConfig::default());
    let gdo0 = Input::new(radio.gdo0.0, InputConfig::default());
    let gdo2 = Input::new(radio.gdo2.0, InputConfig::default());

    let settings: &mut Settings = SETTINGS.init(Settings {
        led_enabled: false,
        wifi_ssid: String::try_from("123").unwrap(),
        min_vol: 1,
        max_vol: 12,
        current_vol: 5,
    });

    spawner.spawn(rf_sensor_task(spi_bus, cs_pin, gdo0, gdo2, settings).unwrap());
}

#[embassy_executor::task]
async fn rf_sensor_task(
    spi_bus: &'static Mutex<CriticalSectionRawMutex, Spi<'static, Async>>,
    cs_pin: Output<'static>,
    gdo0: Input<'static>,
    gdo2: Input<'static>,
    _settings: &'static mut Settings,
) {
    let config = RadioConfig::new()
        .frequency_hz(433_920_000)
        .baud_rate(38_400)
        .modulation(Modulation::Gfsk)
        .tx_power(TxPower::Dbm0);

    let spi = SpiDevice::new(spi_bus, cs_pin);
    //let mut radio = Cc1101::new(spi, gdo0, gdo2).await.unwrap();

    let mut radio = match Cc1101::new(spi, gdo0, gdo2).await {
        Ok(radio) => radio,
        Err(e) => {
            error!(
                "CC1101 initialization failed: {:?}",
                defmt::Debug2Format(&e)
            );
            return;
        }
    };

    radio.configure(&config).await.unwrap();

    loop {
        info!("Sensor task");

        let n = Messaging::new(MessageKind::Error, "Some error during device RF sensor");
        let p = OutboundPackage::Notification(&n);
        Messaging::publish(&p).await;

        Timer::after(Duration::from_secs(5)).await;
    }
}
