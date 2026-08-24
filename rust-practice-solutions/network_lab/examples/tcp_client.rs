use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use tokio::time::timeout;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let stream = timeout(Duration::from_secs(3), TcpStream::connect("127.0.0.1:7000")).await??;

    let (reader, mut writer) = stream.into_split();
    let mut lines = BufReader::new(reader).lines();
    writer.write_all(b"hello\n").await?;
    writer.write_all(b"rust\n").await?;
    writer.shutdown().await?;

    while let Some(line) = lines.next_line().await? {
        println!("server: {line}");
    }
    Ok(())
}
