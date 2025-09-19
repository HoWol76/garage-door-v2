use defmt::info;
use heapless::String;
use esp_hal::gpio::{Input, AnyPin, InputConfig, Pull};
use embassy_time::{Duration, Timer};
use mcutie::Topic::Device;
use mcutie::Publishable as _;


const DEBOUNCE_DURATION: Duration = Duration::from_millis(50);

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum DoorState {
    OPEN,
    CLOSED
}

pub struct Sensor {
    pin: Input<'static>,
    name: String<32>,
}

impl Sensor {
    pub fn new(pin: AnyPin<'static>, name: &str) -> Self {
        Self {
            pin: Input::new(pin, InputConfig::default().with_pull(Pull::Up)),
            name: name.try_into().unwrap(),
        }
    }
  
    /// Check current door state
    pub fn read_state(&self) -> DoorState {
        if self.pin.is_high() {
            DoorState::OPEN
        } else {
            DoorState::CLOSED
        }
    }
  
    pub async fn wait_for_state(&self, desired_state: DoorState) {
        loop {
            if self.read_state() == desired_state {
                break;
            }
            Timer::after(Duration::from_millis(50)).await;
        }
    }
}

#[embassy_executor::task]
pub async fn sensor_monitoring_task(sensor: Sensor) {
    info!("Sensor monitoring task starting");
  
    if sensor.read_state() == DoorState::OPEN {
        Timer::after(DEBOUNCE_DURATION).await;
        let _ = Device(sensor.name.as_str()).with_display("open").publish().await;
        info!("Sensor {} initial state: open", sensor.name);
    }
  
    loop {
        sensor.wait_for_state(DoorState::CLOSED).await;
        Timer::after(DEBOUNCE_DURATION).await;
        let _ = Device(sensor.name.as_str()).with_display("closed").publish().await;
        info!("Sensor {} event: closed", sensor.name);
  
        sensor.wait_for_state(DoorState::OPEN).await;
        Timer::after(DEBOUNCE_DURATION).await;
        let _ = Device(sensor.name.as_str()).with_display("open").publish().await;
        info!("Sensor {} event: open", sensor.name);
    }
}