use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::{connect_async, tungstenite::Message};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (mut socket, response) = connect_async("ws://127.0.0.1:9100").await?;
    println!("handshake status: {}", response.status());
    socket.send(Message::Text("hello".into())).await?;

    while let Some(message) = socket.next().await {
        match message? {
            Message::Text(text) => {
                println!("echo: {text}");
                break;
            }
            Message::Binary(bytes) => println!("binary: {} bytes", bytes.len()),
            Message::Ping(payload) => println!("ping: {} bytes", payload.len()),
            Message::Pong(_) => {}
            Message::Close(frame) => {
                println!("closed: {frame:?}");
                break;
            }
            _ => {}
        }
    }
    Ok(())
}
