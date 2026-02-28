pub mod listener;
pub mod peer;

use crate::comm::events::NodeEvent;
use tokio::sync::mpsc;

pub struct Node {
    sender: mpsc::Sender<NodeEvent>,
}

impl Node {
    pub async fn new(listen_addr: String) -> Self {
        let (tx, rx) = mpsc::channel::<NodeEvent>(30);
        println!("Node starting on {}", listen_addr);

        tokio::spawn(async move { Self::start_node(rx).await });
        tokio::spawn(crate::node::listener::start_listener(
            listen_addr.to_string(),
            tx.clone(),
        ));
        println!("Node running on {}", listen_addr);

        Self { sender: tx }
    }

    pub async fn bootstrap(&self, peer_addr: String) {
        println!("Connecting to {}", peer_addr);
        peer::connect_to_peer(peer_addr, self.sender.clone()).await;
    }

    async fn start_node(mut rx: mpsc::Receiver<NodeEvent>) {
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
}
