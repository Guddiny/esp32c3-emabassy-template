use esp_hal::peripherals::{
    FROM_CPU_INTR0, GPIO0, GPIO1, GPIO4, GPIO8, GPIO9, GPIO10, Peripherals, SPI2, TIMG0,
    USB_DEVICE, WIFI,
};

pub struct LedPin(pub GPIO8<'static>);
pub struct ButtonPin(pub GPIO9<'static>);
pub struct AdcPin(pub GPIO1<'static>);
pub struct RadioCsPin(pub GPIO10<'static>);
pub struct RadioGdo0Pin(pub GPIO4<'static>);
pub struct RadioGdo2Pin(pub GPIO0<'static>);
pub struct RadioSpi(pub SPI2<'static>);
pub struct Wifi(pub WIFI<'static>);
pub struct UsbSerial(pub USB_DEVICE<'static>);

pub struct RadioSpiPins {
    pub cs: RadioCsPin,
    pub gdo0: RadioGdo0Pin,
    pub gdo2: RadioGdo2Pin,
    pub spi: RadioSpi,
}

pub struct Board {
    pub led: LedPin,
    pub button: ButtonPin,
    pub radio: RadioSpiPins,
    pub rtos_timer: TIMG0<'static>,
    pub rtos_irq: FROM_CPU_INTR0<'static>,
    pub wifi: Wifi,
    pub usb: UsbSerial,
}

impl Board {
    pub fn init(peripherals: Peripherals) -> Self {
        Self {
            led: LedPin(peripherals.GPIO8),
            button: ButtonPin(peripherals.GPIO9),
            radio: RadioSpiPins {
                cs: RadioCsPin(peripherals.GPIO10),
                gdo0: RadioGdo0Pin(peripherals.GPIO4),
                gdo2: RadioGdo2Pin(peripherals.GPIO0),
                spi: RadioSpi(peripherals.SPI2),
            },
            wifi: Wifi(peripherals.WIFI),
            rtos_timer: peripherals.TIMG0,
            rtos_irq: peripherals.FROM_CPU_INTR0,
            usb: UsbSerial(peripherals.USB_DEVICE),
        }
    }
}
