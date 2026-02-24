use crate::comm::net_message::NetworkMessage;

#[derive(Debug)]
pub enum NodeEvent {
    PeerConnected(String),
    PeerDisconnected(String),
    Message {
        peer_id: String,
        message: NetworkMessage,
    },
}
