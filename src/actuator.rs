use defmt::info;
use esp_hal::gpio::{Output, AnyPin, OutputConfig, Level};
use embassy_time::{Duration, Timer};

const TOGGLE_PULSE_DURATION: Duration = Duration::from_millis(200);

/// Simple relay actuator that drives a GPIO high for a short pulse.
pub struct Actuator {
    pin: Output<'static>,
}

impl Actuator {
    /// Create a new actuator from a raw pin. Pin is initialised low.
    pub fn new(pin: AnyPin<'static>) -> Self {
        Self {
            pin: Output::new(pin, Level::Low, OutputConfig::default()),
        }
    }
  
    /// Toggle the relay: set pin high for a short pulse, then low again.
    pub async fn toggle(&mut self) {
        info!("Actuator: Toggling relay");
        self.pin.set_high();
        Timer::after(TOGGLE_PULSE_DURATION).await;
        self.pin.set_low();
    }
}