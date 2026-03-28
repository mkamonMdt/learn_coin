use std::fmt::Debug;

use crate::node::peer::Peer;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub enum AlfaProtocols {
    Handshake,
    Unknown,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ProtocolId {
    V0(AlfaProtocols),
}

#[derive(Debug)]
pub enum NodeEvent {
    PeerConnected(Peer, tokio::net::tcp::OwnedWriteHalf),
    PeerDisconnected(Uuid),
    NetworkMessage(NetworkMessage),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NetworkMessage {
    pub peer_id: Uuid,
    pub protocol_id: ProtocolId,
    pub message: Vec<u8>,
}
