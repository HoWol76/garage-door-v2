#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]

use defmt::info;
use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use embassy_net::{DhcpConfig, StackResources};
use esp_hal::clock::CpuClock;
use esp_hal::timer::systimer::SystemTimer;
use esp_hal::timer::timg::TimerGroup;
use esp_wifi::EspWifiController;
use {esp_backtrace as _, esp_println as _};
use mcutie::Topic::Device;

use garage_door_v2::mk_static;
use garage_door_v2::wifi::{connection, net_task, wait_for_connection};
use garage_door_v2::mqtt::mqtt_connection_task;
use garage_door_v2::actuator::Actuator;

extern crate alloc;

// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();

const NUM_SOCKETS: usize = 4;

#[esp_hal_embassy::main]
async fn main(spawner: Spawner) {
    // generator version: 0.5.0

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    esp_alloc::heap_allocator!(size: 80 * 1024);

    let timer0 = SystemTimer::new(peripherals.SYSTIMER);
    esp_hal_embassy::init(timer0.alarm0);

    info!("Embassy initialized!");

    let mut rng = esp_hal::rng::Rng::new(peripherals.RNG);
    let timer1 = TimerGroup::new(peripherals.TIMG0);
    let wifi_init = &*mk_static!(
        EspWifiController<'static>,
        esp_wifi::init(timer1.timer0, rng).expect("Failed to initialize WIFI/BLE controller")
    );
    let (wifi_controller, interfaces) = esp_wifi::wifi::new(wifi_init, peripherals.WIFI)
        .expect("Failed to initialize WIFI controller");
    let wifi_interface = interfaces.sta;
    let net_seed = rng.random() as u64 | ((rng.random() as u64) << 32);
    let net_config = embassy_net::Config::dhcpv4(DhcpConfig::default());

    let (stack, runner) = embassy_net::new(
        wifi_interface,
        net_config,
        mk_static!(StackResources<NUM_SOCKETS>, StackResources::<NUM_SOCKETS>::new()),
        net_seed,
    );

    spawner.spawn(connection(wifi_controller)).ok();
    spawner.spawn(net_task(runner)).ok();
    wait_for_connection(stack).await;

    let subscribed_topics = [
        Device("door1_trigger"),
        Device("door2_trigger"),
    ];
    let mqtt_client: mcutie::McutieBuilder<'static, &'static str,
    mcutie::PublishBytes<'static, &'static str, &'static [u8]>, 2> = 
        mcutie::McutieBuilder::new(
            stack,
            env!("MQTT_DEVICE_TYPE"), 
            env!("MQTT_BROKER")
        )
        .with_subscriptions(subscribed_topics)
        .with_authentication(env!("MQTT_USER"), env!("MQTT_PASSWD"))
        .with_device_id(env!("MQTT_DEVICE_NAME"));

    let (receiver, mqtt_task) = mqtt_client.build();

    let sensor1 = garage_door_v2::sensor::Sensor::new(peripherals.GPIO4.into(), "door1");
    spawner.spawn(garage_door_v2::sensor::sensor_monitoring_task(sensor1)).ok();

    let mut actuator1 = Actuator::new(peripherals.GPIO2.into());

    spawner.spawn(mqtt_connection_task(receiver, actuator1)).ok();
    mqtt_task.run().await;

    loop {
        Timer::after(Duration::from_secs(2)).await;
        info!("Main loop heartbeat");
    }

    // for inspiration have a look at the examples at https://github.com/esp-rs/esp-hal/tree/esp-hal-v1.0.0-rc.0/examples/src/bin
}
