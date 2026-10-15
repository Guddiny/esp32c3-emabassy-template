use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel};
use heapless::Vec;

use crate::config::MAX_PACKET_SIZE;
use crate::ui::funcui::Action;

pub static LED_CHANNEL: Channel<CriticalSectionRawMutex, LedSignal, 4> = Channel::new();
pub static PACKET_CHANNEL: Channel<CriticalSectionRawMutex, Vec<u8, MAX_PACKET_SIZE>, 4> =
    Channel::new();
pub static ACTION_CHANNEL: Channel<CriticalSectionRawMutex, Action, 8> = Channel::new();

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
