use std::fmt::Debug;

use crate::comm::p2p_connection::P2PConnection;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Copy, Debug, Serialize, Deserialize, Eq, Hash, PartialEq)]
pub enum AlfaProtocols {
    Handshake,
    Unknown,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, Eq, Hash, PartialEq)]
pub enum ProtocolId {
    V0(AlfaProtocols),
}

pub enum NodeEvent {
    PeerConnected(P2PConnection),
    PeerDisconnected(Uuid),
    NetworkMessage(NetworkMessage),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NetworkMessage {
    pub peer_id: Uuid,
    pub protocol_id: ProtocolId,
    pub message: Vec<u8>,
}
