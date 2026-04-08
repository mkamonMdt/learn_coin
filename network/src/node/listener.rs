use crate::comm::events::NodeEvent;
use crate::comm::p2p_connection::P2PConnection;
use crate::node::ConnectedPeers;
use tokio::net::TcpListener;
use tokio::sync::mpsc;

pub async fn start_listener(
    addr: String,
    connected_peers: ConnectedPeers,
    event_sender: mpsc::Sender<NodeEvent>,
) {
    let listener = TcpListener::bind(addr.clone())
        .await
        .expect("Failed to bind");

    loop {
        let (stream, _) = listener.accept().await.unwrap();
        let connected_peers = connected_peers.clone();
        let connection = P2PConnection::new(stream, event_sender.clone()).await;
        let id = connection.get_id();

        {
            let mut pending = connected_peers
                .lock()
                .expect("Unrecoverable failure: pending peers mutext poisoned");
            pending.insert(id, connection);
        }
        let _ = event_sender.send(NodeEvent::PeerConnected(id)).await;
    }
}
