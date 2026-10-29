use core::net::Ipv4Addr;
use core::str::FromStr;

use defmt::{error, info};
use embassy_executor::Spawner;
use embassy_net::Stack;
use embassy_time::Duration;
use esp_hal_dhcp_server::simple_leaser::SimpleDhcpLeaser;
use esp_hal_dhcp_server::structs::DhcpServerConfig;

use crate::app::config;

pub async fn start_dhcp_server(spawner: Spawner, stack: Stack<'static>) {
    spawner.spawn(dhcp_server_task(stack).unwrap());
}

#[embassy_executor::task]
async fn dhcp_server_task(stack: Stack<'static>) {
    let ap_ip = Ipv4Addr::from_str(config::WIFI_AP_STATIC_IP).unwrap();
    let [a, b, c, _] = ap_ip.octets();
    let gateways = [ap_ip];
    let dns = [ap_ip];

    let config = DhcpServerConfig {
        ip: ap_ip,
        lease_time: Duration::from_secs(3600),
        gateways: &gateways,
        subnet: None, // Defaults to /24
        dns: &dns,
        use_captive_portal: true,
    };

    let mut leaser = SimpleDhcpLeaser {
        start: Ipv4Addr::new(a, b, c, config::WIFI_AP_GATEWAY_START_IP),
        end: Ipv4Addr::new(a, b, c, config::WIFI_AP_GATEWAY_END_IP),
        leases: Default::default(),
    };

    info!("Starting DHCP server");

    if let Err(e) = esp_hal_dhcp_server::run_dhcp_server(stack, config, &mut leaser).await {
        error!("DHCP server error: {:?}", e);
    }
}
