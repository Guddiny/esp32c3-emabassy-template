use core::str::FromStr;

use defmt::{error, info};
use embassy_executor::Spawner;
use embassy_net::{Runner, Stack, StackResources, StaticConfigV4};
use embassy_time::{Duration, Timer};
use esp_hal::peripherals::WIFI;
use esp_radio::wifi::{
    AuthenticationMethodConfig,
    ap::{AccessPointConfig, EventInfo},
};
use static_cell::StaticCell;

use crate::config;

// One socket per web task plus one UDP socket for the DHCP server.
static STACK_RESOURCES: StaticCell<StackResources<{ config::WEB_STACK_SIZE }>> = StaticCell::new();

pub async fn start_wifi(spawner: Spawner, peripherals_wifi: WIFI<'static>) -> Stack<'static> {
    let ap_ip: core::net::Ipv4Addr =
        core::net::Ipv4Addr::from_str(config::WIFI_AP_STATIC_IP).unwrap();

    let wifi_controller =
        esp_radio::wifi::WifiController::new(peripherals_wifi, Default::default())
            .expect("Failed to initialize Wi-Fi controller");
    let wifi_interface = esp_radio::wifi::Interface::access_point();

    // Ask for an IP address automatically over DHCP, the way a PC does when
    // it joins a network.
    let net_config = embassy_net::Config::ipv4_static(StaticConfigV4 {
        address: embassy_net::Ipv4Cidr::new(ap_ip, 24),
        gateway: Some(ap_ip),
        dns_servers: Default::default(),
    });

    // The TCP/IP stack wants a random seed (used for things like initial sequence numbers).
    // Build a 64-bit seed from the hardware random generator.
    let rng = esp_hal::rng::Rng::new();
    let seed = (rng.random() as u64) << 32 | (rng.random() as u64);

    // Create the network stack. It returns two halves: stack, the handle we use
    // to open sockets and query our IP, and runner, the background worker that
    // actually moves packets and must be polled continuously.
    let (stack, runner) = embassy_net::new(
        wifi_interface,
        net_config,
        STACK_RESOURCES.init(embassy_net::StackResources::new()),
        seed,
    );

    info!("Spawning Wi-Fi Interface...");
    spawner.spawn(wifi_task(wifi_controller).unwrap());
    spawner.spawn(net_task(runner).unwrap());

    info!("wait_config_up");
    stack.wait_config_up().await;
    info!("config_up");

    if let Some(config) = stack.config_v4() {
        info!("Got IP: {}", config.address);
    }

    info!("Network configuration complete. Starting web server.");

    // main hands the stack + LED to the web server task and idles.
    stack
}

#[embassy_executor::task]
async fn wifi_task(mut controller: esp_radio::wifi::WifiController<'static>) {
    const SSID: &str = config::WIFI_AP_SSID;
    const PASSWORD: &str = config::WIFI_AP_PASSW;

    // Describe ourselves as a station (a normal client joining an access point)
    // and apply that configuration to the radio.
    let ap_config = esp_radio::wifi::Config::AccessPoint(
        AccessPointConfig::default()
            .with_ssid(SSID.try_into().unwrap())
            .with_authentication(AuthenticationMethodConfig::Wpa2Personal(
                PASSWORD.try_into().unwrap(),
            )),
    );

    controller.set_config(&ap_config).unwrap();

    info!("Radio configuration set, attempting connection...");

    // In AP mode there is nothing to (re)connect; just log clients joining and
    // leaving. Don't sleep between waits, or events can be missed.
    loop {
        match controller
            .wait_for_access_point_connected_event_async()
            .await
        {
            Ok(EventInfo::Connected(info)) => info!("Client connected: {:?}", info),
            Ok(EventInfo::Disconnected(info)) => info!("Client disconnected: {:?}", info),
            Err(e) => {
                error!("Wi-Fi event error: {:?}", e);
                Timer::after(Duration::from_millis(1000)).await;
            }
        }
    }
}

// Drives the network stack. This call never returns: it loops forever handling
// the low-level packet work so the rest of the program can use TCP.
#[embassy_executor::task]
async fn net_task(mut runner: Runner<'static, esp_radio::wifi::Interface>) -> ! {
    runner.run().await
}
