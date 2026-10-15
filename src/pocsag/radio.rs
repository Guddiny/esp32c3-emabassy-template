use cc1101_embassy::{Cc1101, Modulation, PacketLength, RadioConfig, TxPower};
use embedded_hal::delay::DelayNs;
use embedded_hal_async::digital::Wait;
use embedded_hal_async::spi::SpiDevice;

use crate::pocsag::consts::{BaudRate, tx_power};
use crate::pocsag::encoder;

/// Async POCSAG pager driver for CC1101.
///
/// `SpiError` is the error type produced by the SPI device and must match
/// the `SpiDevice::Error` associated type used by `Cc1101`.
pub struct PagerDriver<SPI, GDO0, GDO2, D, SpiError>
where
    SPI: SpiDevice<Error = SpiError>,
    GDO0: Wait,
    GDO2: Wait,
    D: DelayNs,
{
    radio: Cc1101<SPI, GDO0, GDO2>,
    delay: D,
    freq_hz: u32,
    current_baud: BaudRate,
    tx_power: u8,
    invert: bool,
}

impl<SPI, GDO0, GDO2, D, SpiError> PagerDriver<SPI, GDO0, GDO2, D, SpiError>
where
    SPI: SpiDevice<Error = SpiError>,
    GDO0: Wait,
    GDO2: Wait,
    D: DelayNs,
{
    /// Create a new pager driver.
    pub async fn new(
        spi: SPI,
        gdo0: GDO0,
        gdo2: GDO2,
        delay: D,
        freq_hz: u32,
    ) -> Result<Self, cc1101_embassy::Error<SpiError>> {
        // FIX: explicitly supply all four generic args
        let radio = Cc1101::<SPI, GDO0, GDO2>::new(spi, gdo0, gdo2).await?;

        Ok(Self {
            radio,
            delay,
            freq_hz,
            current_baud: BaudRate::B1200,
            tx_power: tx_power::MAX,
            invert: false,
        })
    }

    pub async fn configure_for_pocsag(
        &mut self,
        baud: BaudRate,
    ) -> Result<(), cc1101_embassy::Error<SpiError>> {
        self.current_baud = baud;

        let config = RadioConfig::new()
            .frequency_hz(self.freq_hz)
            .baud_rate(baud.hz())
            .modulation(Modulation::Fsk2)
            .packet_length(PacketLength::Variable(255))
            .tx_power(TxPower::Dbm10);

        self.radio.configure(&config).await?;
        Ok(())
    }

    pub fn set_tx_power(&mut self, power: u8) {
        self.tx_power = power;
    }

    pub fn set_invert(&mut self, invert: bool) {
        self.invert = invert;
    }

    pub async fn transmit(
        &mut self,
        ric: u32,
        func: u8,
        msg: &[u8],
        baud: BaudRate,
        alpha: bool,
    ) -> Result<(), cc1101_embassy::Error<SpiError>> {
        if baud != self.current_baud {
            self.configure_for_pocsag(baud).await?;
        }

        let mut buffer = [0u32; 512];
        let cw_count = encoder::build_transmit_buffer(ric, func, alpha, msg, baud, &mut buffer);

        if cw_count == 0 {
            return Ok(());
        }

        let byte_count = cw_count * 4;
        let mut bytes = [0u8; 2048];
        for (i, &cw) in buffer[..cw_count].iter().enumerate() {
            bytes[i * 4] = (cw >> 24) as u8;
            bytes[i * 4 + 1] = (cw >> 16) as u8;
            bytes[i * 4 + 2] = (cw >> 8) as u8;
            bytes[i * 4 + 3] = cw as u8;
        }

        self.radio.transmit(&bytes[..byte_count]).await?;
        Ok(())
    }

    pub async fn receive(
        &mut self,
        buf: &mut [u8],
    ) -> Result<cc1101_embassy::ReceivedPacket, cc1101_embassy::Error<SpiError>> {
        self.radio.receive(buf).await
    }

    pub async fn idle(&mut self) -> Result<(), cc1101_embassy::Error<SpiError>> {
        self.radio.idle().await
    }

    pub async fn read_rssi(&mut self) -> Result<i16, cc1101_embassy::Error<SpiError>> {
        self.radio.read_rssi().await
    }
}
