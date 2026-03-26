use crate::node::peer::Peer;

#[derive(Debug)]
pub enum NodeEvent {
    PeerConnected(Peer, tokio::net::tcp::OwnedWriteHalf),
    PeerDisconnected(String),
    NetworkMessage { peer_id: String, message: Vec<u8> },
}
