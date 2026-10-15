use cc1101_embassy::{Cc1101, Modulation, RadioConfig, TxPower};
use defmt::{error, info};
use embassy_embedded_hal::shared_bus::asynch::spi::SpiDevice;
use embassy_executor::Spawner;
use embassy_sync::{blocking_mutex::raw::NoopRawMutex, mutex::Mutex};
use embassy_time::{Duration, Timer};
use esp_hal::{
    Async,
    gpio::{Input, InputConfig, Level, Output, OutputConfig},
    peripherals::{GPIO4, GPIO10, GPIO12, SPI2},
    spi::master::{Config, Spi},
};
use static_cell::StaticCell;

type RadioSpiBus = Spi<'static, Async>;

static RADIO_SPI_BUS: StaticCell<Mutex<NoopRawMutex, RadioSpiBus>> = StaticCell::new();

pub fn start_rf_sensor(
    spawner: Spawner,
    spi_peripheral: SPI2<'static>,
    cs_pin: GPIO4<'static>,
    gdo0_pin: GPIO10<'static>,
    gdo2_pin: GPIO12<'static>,
) {
    let spi = Spi::new(spi_peripheral, Config::default())
        .unwrap()
        .into_async();
    let spi_bus = RADIO_SPI_BUS.init(Mutex::new(spi));
    let cs_pin = Output::new(cs_pin, Level::High, OutputConfig::default());
    let gdo0 = Input::new(gdo0_pin, InputConfig::default());
    let gdo2 = Input::new(gdo2_pin, InputConfig::default());

    spawner.spawn(rf_sensor_task(spi_bus, cs_pin, gdo0, gdo2).unwrap());
}

#[embassy_executor::task]
async fn rf_sensor_task(
    spi_bus: &'static Mutex<NoopRawMutex, RadioSpiBus>,
    cs_pin: Output<'static>,
    gdo0: Input<'static>,
    gdo2: Input<'static>,
) {
    let config = RadioConfig::new()
        .frequency_hz(433_920_000)
        .baud_rate(38_400)
        .modulation(Modulation::Gfsk)
        .tx_power(TxPower::Dbm0);

    let spi = SpiDevice::new(spi_bus, cs_pin);
    // let mut radio = Cc1101::new(spi, gdo0, gdo2).await.unwrap();

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
        Timer::after(Duration::from_secs(5)).await;
    }
}
