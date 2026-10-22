use cc1101_embassy::{Modulation, RadioConfig, TxPower};
use defmt::info;
use embassy_embedded_hal::shared_bus::asynch::spi::SpiDevice;
use embassy_executor::Spawner;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, mutex::Mutex};
use embassy_time::{Duration, Timer};
use esp_hal::{
    Async,
    gpio::{Input, InputConfig, Level, Output, OutputConfig},
    peripherals::{GPIO0, GPIO4, GPIO10, SPI2},
    spi::master::{Config, Spi},
};
use heapless::String;
use static_cell::StaticCell;

use crate::notifications::{Notification, NotificationKind};
use crate::state::Settings;

type RadioSpiBus = Spi<'static, Async>;

static RADIO_SPI_BUS: StaticCell<Mutex<CriticalSectionRawMutex, RadioSpiBus>> = StaticCell::new();
static SETTINGS: StaticCell<Settings> = StaticCell::new();

pub fn start_rf_sensor(
    spawner: Spawner,
    spi_peripheral: SPI2<'static>,
    cs_pin: GPIO10<'static>,
    gdo0_pin: GPIO4<'static>,
    gdo2_pin: GPIO0<'static>,
) {
    // #define SS_CS_PIN 10
    // #define SCK_PIN 6
    // #define MOSI_PIN 7
    // #define MISO_PIN 2
    // #define GD0 4
    // #define GD2 0

    let spi = Spi::new(spi_peripheral, Config::default())
        .unwrap()
        .into_async();
    let spi_bus = RADIO_SPI_BUS.init(Mutex::new(spi));
    let cs_pin = Output::new(cs_pin, Level::High, OutputConfig::default());
    let gdo0 = Input::new(gdo0_pin, InputConfig::default());
    let gdo2 = Input::new(gdo2_pin, InputConfig::default());

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
    spi_bus: &'static Mutex<CriticalSectionRawMutex, RadioSpiBus>,
    cs_pin: Output<'static>,
    gdo0: Input<'static>,
    gdo2: Input<'static>,
    settings: &'static mut Settings,
) {
    let config = RadioConfig::new()
        .frequency_hz(433_920_000)
        .baud_rate(38_400)
        .modulation(Modulation::Gfsk)
        .tx_power(TxPower::Dbm0);

    let spi = SpiDevice::new(spi_bus, cs_pin);
    // let mut radio = Cc1101::new(spi, gdo0, gdo2).await.unwrap();

    // let mut radio = match Cc1101::new(spi, gdo0, gdo2).await {
    //     Ok(radio) => radio,
    //     Err(e) => {
    //         error!(
    //             "CC1101 initialization failed: {:?}",
    //             defmt::Debug2Format(&e)
    //         );
    //         return;
    //     }
    // };

    // radio.configure(&config).await.unwrap();

    loop {
        info!("Sensor task");

        let n = Notification::new(
            NotificationKind::Error,
            "Some error during device RF sensor",
        );
        let p = crate::notifications::OutboundPackage::Notification(&n);
        Notification::publish(&p).await;

        Timer::after(Duration::from_secs(5)).await;
    }
}
