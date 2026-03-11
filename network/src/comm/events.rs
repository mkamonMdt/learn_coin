use tokio::sync::oneshot;

use crate::comm::net_message::NetworkMessage;
use crate::node::peer::Peer;
use crate::protocols::peer_handshake::PeerHandshake;

#[derive(Debug)]
pub enum NodeEvent {
    PeerHandshake(PeerHandshake, oneshot::Sender<()>),
    PeerConnected(Peer),
    PeerDisconnected(String),
    Message {
        peer_id: String,
        message: NetworkMessage,
    },
}
