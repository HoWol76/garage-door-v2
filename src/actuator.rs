use defmt::info;
use esp_hal::gpio::{Output, AnyPin, OutputConfig, Level};
use embassy_time::{Duration, Timer};

const TOGGLE_PULSE_DURATION: Duration = Duration::from_millis(200);

pub struct Actuator {
    pin: Output<'static>,
}

impl Actuator {
    pub fn new(pin: AnyPin<'static>) -> Self {
        Self {
            pin: Output::new(pin, Level::Low, OutputConfig::default()),
        }
    }
  
    pub async fn toggle(&mut self) {
        info!("Actuator: Toggling relay");
        self.pin.set_high();
        Timer::after(TOGGLE_PULSE_DURATION).await;
        self.pin.set_low();
    }
}

// /// Simple actuator toggle task that can be spawned
// #[embassy_executor::task]
// pub async fn actuator_toggle_task(mut actuator: Actuator) {
//     actuator.toggle().await;
// }