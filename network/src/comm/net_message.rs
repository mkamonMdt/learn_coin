use serde::{Deserialize, Serialize};
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum NetworkMessage {
    Ping,
    Pong,
}

pub async fn send_message(mut stream: TcpStream, msg: &NetworkMessage) -> std::io::Result<()> {
    let bytes = bincode::serialize(msg).unwrap();
    let len = bytes.len() as u32;

    stream.write_u32(len).await?;
    stream.write_all(&bytes).await?;
    Ok(())
}
