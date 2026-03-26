use crate::node::peer::Peer;
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub enum NodeEvent {
    PeerConnected(Peer, tokio::net::tcp::OwnedWriteHalf),
    PeerDisconnected(String),
    NetworkMessage(NetworkMessage),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NetworkMessage {
    pub peer_id: String,
    pub message: Vec<u8>,
}
