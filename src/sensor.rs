use defmt::{info, Format};
use esp_hal::gpio::{Input, AnyPin, InputConfig, Pull};
use embassy_time::{Duration, Timer};
use mcutie::Topic::Device;
use mcutie::Publishable as _;

const DEBOUNCE_DURATION: Duration = Duration::from_millis(50);
const POLL_INTERVAL: Duration = Duration::from_millis(50);

#[derive(Copy, Clone, Debug, PartialEq, Format)]
pub enum DoorState {
    Open,
    Closed
}

impl core::fmt::Display for DoorState {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            DoorState::Open => write!(f, "open"),
            DoorState::Closed => write!(f, "closed"),
        }
    }
}

// Convert GPIO high/low to DoorState
// Assuming HIGH = True = Open.
impl From<bool> for DoorState {
    fn from(value: bool) -> Self {
        if value {
            DoorState::Open
        } else {
            DoorState::Closed
        }
    }
}

/// Simple door sensor that reads a GPIO input pin.
pub struct Sensor {
    pin: Input<'static>,
    name: &'static str,
}

impl Sensor {
    /// Create a new sensor from a raw pin. Pin is initialised with pull-up.
    /// name is stored to be used in MQTT topic.
    pub fn new(pin: AnyPin<'static>, name: &'static str) -> Self {
        Self {
            pin: Input::new(pin, InputConfig::default().with_pull(Pull::Up)),
            name: name,
        }
    }
  
    /// Check current door state
    pub fn read_state(&self) -> DoorState {
        self.pin.is_high().into()
    }
  
    /// Wait until door reaches desired state
    pub async fn wait_for_state(&self, desired_state: DoorState) {
        loop {
            if self.read_state() == desired_state {
                break;
            }
            Timer::after(POLL_INTERVAL).await;
        }
    }

    /// Wait until door state changes, then debounce
    pub async fn wait_for_change(&self) {
        let initial_state = self.read_state();
        loop {
            if self.read_state() != initial_state {
                break;
            }
            Timer::after(POLL_INTERVAL).await;
        }
        Timer::after(DEBOUNCE_DURATION).await;
    }
}

/// Sensor monitoring task that can be spawned.
/// Publishes state changes to MQTT.
#[embassy_executor::task]
pub async fn sensor_monitoring_task(sensor: Sensor) {
    info!("Sensor monitoring task starting");
    Timer::after(Duration::from_secs(1)).await; // wait for other tasks to initialize
    loop {
        let state = sensor.read_state();
        info!("Sensor state changed: {}", state);
        let _ = Device(sensor.name)
            .with_display(state)
            .publish()
            .await;
        sensor.wait_for_change().await;
    }
}