use defmt::{error, info};
use embassy_executor::Spawner;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;
use embedded_io::Error;
use embedded_io_async::{Read, Write};
use esp_hal::Async;
use esp_hal::usb::usb_serial_jtag::{UsbSerialJtag, UsbSerialJtagRx, UsbSerialJtagTx};
use heapless::Vec;
use wake::{Decode, Encode, Packet};

use crate::board::board::UsbSerial;
use crate::services::led::{LED_CHANNEL, LedSignal};

pub static USB_CHANNEL: Channel<CriticalSectionRawMutex, Packet, 10> = Channel::new();

pub fn start_usb_driver(spawner: Spawner, usb: UsbSerial) {
    let (usb_rx, usb_tx) = UsbSerialJtag::new(usb.0).into_async().split();

    spawner.spawn(usb_read_task(usb_rx).unwrap());
    spawner.spawn(usb_write_task(usb_tx).unwrap());
    info!("USB task spawned");
}

#[embassy_executor::task]
async fn usb_read_task(mut usb: UsbSerialJtagRx<'static, Async>) {
    let mut line_buf = [0u8; 128];

    loop {
        match usb.read(&mut line_buf).await {
            Ok(0) => {}
            Ok(n) => {
                let inc = line_buf[..n].decode().unwrap();
                LED_CHANNEL.send(LedSignal::OK).await;
                USB_CHANNEL.send(inc).await;
            }
            Err(_) => error!("Error to read USB data"),
        }
    }
}

#[embassy_executor::task]
async fn usb_write_task(mut usb: UsbSerialJtagTx<'static, Async>) {
    loop {
        let pkg = USB_CHANNEL.receive().await;
        let out = pkg.encode().unwrap();
        let r = Write::write(&mut usb, &out).await;
        match r {
            Ok(_) => {}
            Err(_) => LED_CHANNEL.send(LedSignal::Error).await,
        }
    }
}
