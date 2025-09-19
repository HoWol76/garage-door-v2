use defmt::info;
use mcutie::{McutieReceiver, MqttMessage, Topic::{self, DeviceType, General, Device}};
use crate::actuator::Actuator;

#[embassy_executor::task]
pub async fn mqtt_connection_task(receiver: McutieReceiver, mut actuator1: Actuator) {
    loop {
        let msg = receiver.receive().await;
        if let MqttMessage::Publish ( Topic::Device(ref t), ref payload ) = msg {
            if let Ok(s) = core::str::from_utf8(&payload) {
                info!("Received device: {} <- {}", t.as_str(), s);
                if t.as_str() == "door1_trigger" && s == "activate" {
                    actuator1.toggle().await;
                }
            }
        }
    }
}