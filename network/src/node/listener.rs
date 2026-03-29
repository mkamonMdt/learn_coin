use crate::comm::events::ProtocolId;
use crate::comm::p2p_connection::P2PConnection;
use crate::protocols::peer_handshake::accept_protocol;
use crate::{comm::events::NodeEvent, node::peer::Peer};
use tokio::{net::TcpListener, sync::mpsc};

pub async fn start_listener(local_peer: Peer, addr: String, node_tx: mpsc::Sender<NodeEvent>) {
    let listener = TcpListener::bind(addr.clone())
        .await
        .expect("Failed to bind");

    loop {
        let (stream, _) = listener.accept().await.unwrap();
        let node_tx = node_tx.clone();
        let local_peer = local_peer.clone();
        tokio::spawn(async move {
            let conneciton = P2PConnection::new(stream).await;
            let protocol_id = ProtocolId::V0(crate::comm::events::AlfaProtocols::Handshake);
            let handle = conneciton.open_protocol(protocol_id).await;
            let _ = node_tx.send(NodeEvent::PeerConnected(conneciton)).await;

            accept_protocol(local_peer, handle).await;
        });
    }
}
