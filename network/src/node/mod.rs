pub mod listener;
pub mod peer;

use crate::comm::events::NodeEvent;
use crate::node::peer::Peer;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;
use tokio::sync::mpsc;

pub struct Node {
    sender: mpsc::Sender<NodeEvent>,
    peers: Arc<Mutex<HashMap<String, Peer>>>,
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

        Self {
            sender: tx,
            peers: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn bootstrap(&self, peer_addr: String) {
        println!("Connecting to {}", peer_addr);
        match peer::connect_to_peer(peer_addr.clone(), self.sender.clone()).await {
            Ok(_) => {
                let mut peers = self
                    .peers
                    .lock()
                    .expect("Unrecoverable failure: peers mutext poisoned");
                peers.insert(peer_addr.clone(), Peer { addr: peer_addr });
            }
            Err(e) => println!("Network error: {:#?}", e),
        }
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
