use defmt::info;
use mcutie::{McutieReceiver, MqttMessage, Topic::Device};
use crate::actuator::Actuator;

const ACTIVATION_KEYWORD: &str = "fire";
const TRIGGER_TOPIC_SUFFIX: &str = "_trigger";

#[embassy_executor::task]
pub async fn mqtt_connection_task(receiver: McutieReceiver, mut actuators: [Actuator; 1]) {
    loop {
        let msg = receiver.receive().await;
        if let MqttMessage::Publish ( Device(ref t), ref payload ) = msg {
            if let Ok(s) = core::str::from_utf8(&payload) {
                info!("Received device: {} <- {}", t.as_str(), s);
                if s == ACTIVATION_KEYWORD && t.as_str().ends_with(TRIGGER_TOPIC_SUFFIX) {
                    for actuator in actuators.iter_mut() {
                        if t.as_str().starts_with(actuator.name()) {
                            actuator.toggle().await;
                        }
                    }
                }
            }
        }
    }
}