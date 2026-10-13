use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel};

pub static LED_CHANNEL: Channel<CriticalSectionRawMutex, u32, 4> = Channel::new();