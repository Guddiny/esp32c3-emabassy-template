use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel};

pub static LED_CHANNEL: Channel<CriticalSectionRawMutex, LedSignal, 4> = Channel::new();

#[derive(Clone, Copy, Debug)]
pub enum LedSignal {
    OK,
    Error
}

impl LedSignal {
    pub fn blink_count(self) -> u8 
    {
        match self {
            Self::OK => 1,
            Self::Error => 4
        }
    }
}
