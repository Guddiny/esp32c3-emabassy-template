pub const WIFI_AP_SSID: &str = "esp32c3m";
pub const WIFI_AP_PASSW: &str = "12345678";

pub const WIFI_AP_STATIC_IP: &str = "192.168.4.1";
pub const WIFI_AP_GATEWAY_START_IP: u8 = 100;
pub const WIFI_AP_GATEWAY_END_IP: u8 = 115;

pub const HTTP_SERVER_PORT: u16 = 80;
pub const WEB_TASK_POOL_SIZE: usize = 3;
pub const DHCP_SOCKET_SIZE: usize = 1;
pub const MAX_CONCURRENT_WEBSOCKETS: usize = 2;
pub const HEADROOM: usize = 1;
pub const WEB_STACK_SIZE: usize =
    WEB_TASK_POOL_SIZE + DHCP_SOCKET_SIZE + MAX_CONCURRENT_WEBSOCKETS + HEADROOM;
