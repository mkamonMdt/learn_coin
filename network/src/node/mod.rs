pub mod listener;
pub mod peer;

use crate::comm::events::NodeEvent;
use crate::node::peer::Peer;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;
use tokio::net::tcp::OwnedWriteHalf;
use tokio::sync::mpsc;
use uuid::Uuid;

#[derive(Clone)]
struct Peers {
    connected: Arc<Mutex<HashMap<String, Peer>>>,
    pending: Arc<Mutex<HashMap<String, PendingPeer>>>,
}

impl Peers {
    fn new() -> Self {
        Self {
            connected: Arc::new(Mutex::new(HashMap::new())),
            pending: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

/// TODO: remove that. move writer to Peer context
pub struct PendingPeer {
    addr: String,
    writer: OwnedWriteHalf,
}

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

        let peers = Peers::new();
        {
            let value = peers.clone();
            tokio::spawn(async move { Self::start_node(value.clone(), rx).await });
        }
        tokio::spawn(crate::node::listener::start_listener(
            local_peer.clone(),
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
                NodeEvent::PeerConnected(peer, writer) => {
                    let mut pending = peers
                        .pending
                        .lock()
                        .expect("Unrecoverable failure: pending peers mutext poisoned");
                    pending.insert(
                        peer.addr.clone(),
                        PendingPeer {
                            addr: peer.addr,
                            writer,
                        },
                    );
                }
                NodeEvent::PeerDisconnected(id) => {
                    println!("Peer disconnected: {}", id);
                }
                NodeEvent::NetworkMessage { peer_id, message } => {
                    println!("Message from {}:{:?}: ", peer_id, message);
                }
            }
        }
    }
}
