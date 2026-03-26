use crate::node::peer::handle_peer;
use crate::protocols::peer_handshake::accept_protocol;
use crate::{comm::events::NodeEvent, node::peer::Peer};
use tokio::{net::TcpListener, sync::mpsc};

pub async fn start_listener(local_peer: Peer, addr: String, node_tx: mpsc::Sender<NodeEvent>) {
    let listener = TcpListener::bind(addr.clone())
        .await
        .expect("Failed to bind");

    loop {
        let (stream, addr) = listener.accept().await.unwrap();
        let node_tx = node_tx.clone();
        let local_peer = local_peer.clone();
        tokio::spawn(async move {
            let (peer, reader, writer) = accept_protocol(local_peer, stream).await.unwrap();
            let _ = node_tx.send(NodeEvent::PeerConnected(peer, writer)).await;
            let _ = handle_peer(reader, node_tx, addr.to_string()).await;
        });
    }
}
