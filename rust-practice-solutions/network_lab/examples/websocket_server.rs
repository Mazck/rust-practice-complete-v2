use futures_util::{SinkExt, StreamExt};
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::accept_async;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("127.0.0.1:9100").await?;
    println!("WebSocket server listening on 127.0.0.1:9100");

    loop {
        let (stream, peer) = listener.accept().await?;
        tokio::spawn(async move {
            if let Err(error) = handle(stream).await {
                eprintln!("{peer}: {error}");
            }
        });
    }
}

async fn handle(stream: TcpStream) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut socket = accept_async(stream).await?;
    while let Some(message) = socket.next().await {
        let message = message?;
        if message.is_close() {
            break;
        }
        socket.send(message).await?;
    }
    Ok(())
}
