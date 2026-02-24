pub mod listener;
pub mod peer;

use tokio::sync::mpsc;

use crate::comm::events::NodeEvent;

pub async fn start_node(mut rx: mpsc::Receiver<NodeEvent>) {
    while let Some(event) = rx.recv().await {
        match event {
            NodeEvent::PeerConnected(id) => {
                println!("Peer connected: {}", id);
            }
            NodeEvent::PeerDisconnected(id) => {
                println!("Peer disconnected: {}", id);
            }
            NodeEvent::Message { peer_id, message } => {
                println!("Message from {}: {:?}", peer_id, message);
            }
        }
    }
}
