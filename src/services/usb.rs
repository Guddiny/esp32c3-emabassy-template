use defmt::{error, info};
use embassy_executor::Spawner;
use embedded_io_async::Read;
use esp_hal::Async;
use esp_hal::usb::usb_serial_jtag::UsbSerialJtag;

use crate::board::board::UsbSerial;

pub fn start_usb_driver(spawner: Spawner, usb: UsbSerial) {
    let usb = UsbSerialJtag::new(usb.0).into_async();

    spawner.spawn(usb_task(usb).unwrap());
}

#[embassy_executor::task]
async fn usb_task(mut usb: UsbSerialJtag<'static, Async>) {
    let mut line_buf = [0u8; 128];
    // let mut line_len = 0;
    // let mut byte_buf = [0u8; 1];

    loop {
        match usb.read(&mut line_buf).await {
            Ok(0) => {}
            Ok(n) => {
                for b in &line_buf[..n] {
                    info!("B {}", b);
                }
            }
            Err(_) => error!("Error to read USB data"),
        }
    }
}
