use heapless::String;
use serde::{Deserialize, Serialize};

#[derive(Default, Serialize, Deserialize)]
pub struct Settings {
    pub led_enabled: bool,

    pub wifi_ssid: String<32>,

    pub min_vol: u8,
    pub max_vol: u8,
    pub current_vol: u8,
}

#[derive(Default, Serialize, Deserialize)]
pub struct State {
    pub is_running: bool,
}

pub struct AppState {
    pub settings: Settings,
    pub state: State,
}
