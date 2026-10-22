use defmt::{error, info};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel};
use heapless::{String, Vec};
use serde::Serialize;

use crate::{config, shared::PACKET_CHANNEL, ui::funcui::Widget};

pub static NOTIFICATIONS_CHANNEL: Channel<CriticalSectionRawMutex, Notification, 4> =
    Channel::new();

#[derive(Serialize)]
pub enum LogLevel {
    #[serde(rename = "info")]
    Info,
    #[serde(rename = "warning")]
    Warning,
    #[serde(rename = "error")]
    Error,
}

#[derive(Serialize)]
pub struct Log {
    level: LogLevel,
    message: String<32>,
}

#[derive(Serialize)]
#[allow(clippy::large_enum_variant)]
pub enum OutboundPackage<'a> {
    #[serde(rename = "data")]
    Data(&'a [Widget]),
    #[serde(rename = "notification")]
    Notification(&'a Notification),
    #[serde(rename = "log")]
    Log(&'a Log),
}

#[derive(Serialize)]
pub struct Notification {
    #[serde(rename = "kind")]
    pub kind: NotificationKind,
    #[serde(rename = "message")]
    pub message: String<64>,
}

#[derive(Serialize)]
pub enum NotificationKind {
    #[serde(rename = "info")]
    Info,
    #[serde(rename = "warning")]
    Warning,
    #[serde(rename = "error")]
    Error,
}

impl Notification {
    pub fn new(kind: NotificationKind, message: &str) -> Self {
        Self {
            kind,
            message: String::try_from(message).unwrap(),
        }
    }

    pub async fn publish(notification: &OutboundPackage<'_>) {
        let packet = match Notification::serialize_package(notification) {
            Ok(packet) => packet,
            Err(err) => {
                error!("JSON serialize failed: {:?}", defmt::Debug2Format(&err));
                return;
            }
        };

        info!("Publishing notification: {} bytes", packet.len());

        PACKET_CHANNEL.send(packet).await;
    }

    fn serialize_package(
        package: &OutboundPackage,
    ) -> Result<Vec<u8, { config::MAX_PACKET_SIZE }>, serde_json_core::ser::Error> {
        let mut packet = Vec::<u8, { config::MAX_PACKET_SIZE }>::new();

        packet
            .resize(config::MAX_PACKET_SIZE, 0)
            .expect("packet length equals its capacity");

        let len = serde_json_core::to_slice(package, packet.as_mut_slice())?;
        packet.truncate(len);

        Ok(packet)
    }
}
