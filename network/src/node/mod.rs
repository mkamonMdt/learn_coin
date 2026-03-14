pub mod listener;
pub mod peer;

use crate::comm::events::NodeEvent;
use crate::node::peer::Peer;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;
use tokio::sync::mpsc;
use uuid::Uuid;

type Peers = Arc<Mutex<HashMap<String, Peer>>>;

pub struct Node {
    sender: mpsc::Sender<NodeEvent>,
    peers: Peers,
    local_peer: Peer,
}

impl Node {
    pub async fn new(listen_addr: String) -> Self {
        let local_peer = Peer {
            addr: listen_addr.clone(),
            id: Uuid::new_v4(),
        };
        let (tx, rx) = mpsc::channel::<NodeEvent>(30);
        println!("Node starting on {}", listen_addr);

        let peers: Arc<Mutex<HashMap<String, Peer>>> = Arc::new(Mutex::new(HashMap::new()));
        {
            let value = peers.clone();
            tokio::spawn(async move { Self::start_node(value.clone(), rx).await });
        }
        tokio::spawn(crate::node::listener::start_listener(
            listen_addr.to_string(),
            tx.clone(),
        ));
        println!("Node running on {}", listen_addr);

        Self {
            sender: tx,
            peers,
            local_peer,
        }
    }

    pub async fn bootstrap(&self, peer_addr: String) {
        println!("Connecting to {}", peer_addr);
        match peer::connect_to_peer(
            peer_addr.clone(),
            self.sender.clone(),
            self.local_peer.clone(),
        )
        .await
        {
            Ok(()) => {}
            Err(e) => println!("Network error: {:#?}", e),
        }
    }

    async fn start_node(peers: Peers, mut rx: mpsc::Receiver<NodeEvent>) {
        while let Some(event) = rx.recv().await {
            match event {
                NodeEvent::PeerConnected(peer) => {
                    println!("Peer connected: {:?}", peer);
                    let mut peers = peers
                        .lock()
                        .expect("Unrecoverable failure: peers mutext poisoned");
                    peers.insert(peer.addr.clone(), peer);
                }
                NodeEvent::PeerDisconnected(id) => {
                    println!("Peer disconnected: {}", id);
                }
                NodeEvent::Message { peer_id, message } => {
                    println!("Message from {}: {:?}", peer_id, message);
                }
                NodeEvent::PeerHandshake(_peer_handshake, _sender) => todo!(),
            }
        }
    }
}
