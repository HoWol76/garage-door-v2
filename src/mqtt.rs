use defmt::info;

// #[embassy_executor::task]
// pub async fn mqtt_connection_task(receiver: mcutie::McutieReceiver<'static, &'static str,
//     mcutie::PublishBytes<'static, &'static str, &'static [u8]>>) {
//     info!("MQTT connection task starting");
//     loop {
//         if let Some(msg) = receiver.recv().await {
//             info!("Received MQTT message on topic {}: {:?}", msg.topic(), msg.payload());
//         }
//     }
// }