use crate::protocols::peer_handshake::accept_protocol;
use crate::protocols::peer_listener::listen_peer;
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
            let (reader, writer) = stream.into_split();
            let (peer, reader, writer) = accept_protocol(local_peer, reader, writer).await.unwrap();
            let peer_id = peer.id;
            let _ = node_tx.send(NodeEvent::PeerConnected(peer, writer)).await;
            let _ = listen_peer(reader, node_tx, peer_id).await;
        });
    }
}
