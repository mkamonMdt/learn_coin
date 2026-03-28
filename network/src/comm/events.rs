use crate::node::peer::Peer;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug)]
pub enum NodeEvent {
    PeerConnected(Peer, tokio::net::tcp::OwnedWriteHalf),
    PeerDisconnected(Uuid),
    NetworkMessage(NetworkMessage),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NetworkMessage {
    pub peer_id: Uuid,
    pub message: Vec<u8>,
}
