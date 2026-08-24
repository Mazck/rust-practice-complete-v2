use rumqttc::{AsyncClient, Event, MqttOptions, Packet, QoS};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut options = MqttOptions::new("rust-network-verify", "127.0.0.1", 1883);
    options.set_keep_alive(Duration::from_secs(30));

    let (client, mut eventloop) = AsyncClient::new(options, 32);
    client
        .subscribe("demo/temperature", QoS::AtLeastOnce)
        .await?;
    client
        .publish(
            "demo/temperature",
            QoS::AtLeastOnce,
            false,
            br#"{"value": 25.5, "unit": "C"}"#,
        )
        .await?;

    for _ in 0..3 {
        match eventloop.poll().await {
            Ok(Event::Incoming(Packet::Publish(message))) => {
                println!("{}: {:?}", message.topic, message.payload);
            }
            Ok(event) => println!("event: {event:?}"),
            Err(error) => {
                eprintln!("event loop error: {error}");
                break;
            }
        }
    }
    Ok(())
}
