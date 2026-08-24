use std::time::Duration;
use tokio::net::UdpSocket;
use tokio::time::timeout;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let server = UdpSocket::bind("127.0.0.1:0").await?;
    let server_addr = server.local_addr()?;
    let client = UdpSocket::bind("127.0.0.1:0").await?;
    client.connect(server_addr).await?;

    client.send(b"ping").await?;
    let mut buffer = [0_u8; 2048];
    let (size, peer) = timeout(Duration::from_secs(1), server.recv_from(&mut buffer)).await??;
    server.send_to(&buffer[..size], peer).await?;

    let reply_size = timeout(Duration::from_secs(1), client.recv(&mut buffer)).await??;
    assert_eq!(&buffer[..reply_size], b"ping");
    println!("UDP echo: {:?}", &buffer[..reply_size]);
    Ok(())
}
