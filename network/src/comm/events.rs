use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub enum NodeEvent {
    PeerConnected(Uuid),
    PeerDisconnected(Uuid),
    NetworkMessage(NetworkMessage),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NetworkMessage {
    pub peer_id: Uuid,
    pub protocol_id: u16,
    pub message: Vec<u8>,
}
