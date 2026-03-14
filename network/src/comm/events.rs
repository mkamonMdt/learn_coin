use crate::comm::net_message::NetworkMessage;
use crate::node::peer::Peer;
use crate::protocols::peer_handshake::PeerHandshake;
use tokio::sync::oneshot;

#[derive(Debug)]
pub enum NodeEvent {
    PeerHandshake(PeerHandshake, oneshot::Sender<PeerHandshake>),
    PeerConnected(Peer),
    PeerDisconnected(String),
    Message {
        peer_id: String,
        message: NetworkMessage,
    },
}
