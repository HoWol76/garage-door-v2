use defmt::info;
use embassy_net::{Stack, Runner};
use embassy_time::{Duration, Timer};
use esp_wifi::wifi::{Configuration, ClientConfiguration, WifiController, WifiDevice, WifiEvent, WifiState};

const SSID: &str = env!("WIFI_SSID");
const PASSWORD: &str = env!("WIFI_PASSWD");

pub async fn wait_for_connection(stack: Stack<'_>) {
    info!("Waiting for link up...");
    loop {
        if stack.is_link_up() {
            info!("Link is up!");
            break;
        }
        Timer::after(Duration::from_millis(500)).await;
    }

    info!("Waiting for IP address...");
    loop {
        if let Some(config) = stack.config_v4() {
            info!("Got IP address: {}", config.address);
            break;
        }
        Timer::after(Duration::from_millis(500)).await;
    }
}

#[embassy_executor::task]
pub async fn connection(mut controller: WifiController<'static>) {
    info!("Device capabilities: {:?}", controller.capabilities());
    info!("Start connection...");
    loop {
        if matches!(esp_wifi::wifi::wifi_state(), WifiState::StaConnected) {
            // wait until disconnected
            controller.wait_for_event(WifiEvent::StaDisconnected).await;
            info!("WiFi disconnected");
            Timer::after(Duration::from_secs(5)).await;
        }

        if ! matches!(controller.is_started(), Ok(true)) {
            let client_config = Configuration::Client( ClientConfiguration {
                ssid: SSID.into(),
                password: PASSWORD.into(),
                ..Default::default()
            });
            controller.set_configuration(&client_config).unwrap();
            info!("Starting WiFi in client mode...");
            controller.start_async().await.unwrap();
            info!("WiFi started");
        }
        info!("Connecting to WiFi...");
        match controller.connect_async().await {
            Ok(_) => {
                info!("Connected to WiFi");
            }
            Err(e) => {
                info!("Failed to connect to WiFi: {:?}", e);
                Timer::after(Duration::from_secs(5)).await;
                continue;
            }
        }
    }
}

#[embassy_executor::task]
pub async fn net_task(mut runner: Runner<'static, WifiDevice<'static>>) {
    info!("Starting network stack...");
    runner.run().await;
}