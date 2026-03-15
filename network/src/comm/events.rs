use crate::comm::net_message::NetworkMessage;
use crate::node::connection::UnverifiedConnection;
use crate::node::peer::Peer;
use crate::protocols::peer_handshake::PeerHandshake;
use tokio::sync::oneshot;

#[derive(Debug)]
pub enum NodeEvent {
    PeerConnection(PeerConnectionEvent),
    PeerDisconnected(String),
    Message {
        peer_id: String,
        message: NetworkMessage,
    },
}

#[derive(Debug)]
pub enum PeerConnectionEvent {
    IntializingConnection {
        inbound: UnverifiedConnection,
        local_peer: Peer,
    },
    IncommingConnection(UnverifiedConnection),
    PeerHandshake(PeerHandshake, oneshot::Sender<PeerHandshake>),
    PeerConnected(Peer),
}
