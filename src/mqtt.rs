use defmt::info;
use mcutie::{McutieReceiver, MqttMessage, Topic::{self, DeviceType, General, Device}};

#[embassy_executor::task]
pub async fn mqtt_connection_task(receiver: McutieReceiver) {
    loop {
        let msg = receiver.receive().await;
        match msg {
            MqttMessage::Connected => info!("MQTT connected"),
            MqttMessage::Disconnected => info!("MQTT disconnected"),
            MqttMessage::Publish ( topic, payload ) => {
                if let Ok(s) = core::str::from_utf8(&payload) {
                    match topic {
                        DeviceType(t) => info!("Received device type: {} <- {}", t.as_str(), s),
                        Device(t) => info!("Received device: {} <- {}", t.as_str(), s),
                        General(t) => info!("Received general message: {} <- {}", t.as_str(), s),
                    }
                }
            }
            MqttMessage::HomeAssistantOnline => info!("Home Assistant is online"),
        }
    }
}