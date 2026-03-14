use serde::{Deserialize, Serialize};
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;

use crate::protocols::peer_handshake::PeerHandshake;

#[derive(Serialize, Deserialize, Debug)]
pub enum NetworkMessage {
    Handshake(PeerHandshake),
}

pub async fn send_message(stream: &mut TcpStream, msg: &NetworkMessage) -> std::io::Result<()> {
    let bytes = bincode::serialize(msg).unwrap();
    let len = bytes.len() as u32;

    stream.write_u32(len).await?;
    stream.write_all(&bytes).await?;
    Ok(())
}
