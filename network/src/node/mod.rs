pub mod listener;
pub mod peer;

use crate::comm::events::NetworkMessage;
use crate::comm::events::NodeEvent;
use crate::comm::p2p_connection::P2PConnection;
use crate::node::peer::Peer;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;
use tokio::sync::mpsc;
use uuid::Uuid;

type ConnectedPeers = Arc<Mutex<HashMap<Uuid, P2PConnection>>>;

pub struct Node {
    sender: mpsc::Sender<NodeEvent>,
    connected_peers: ConnectedPeers,
    local_peer: Peer,
}

impl Node {
    pub async fn new(listen_addr: String) -> Self {
        let local_peer = Peer { id: Uuid::new_v4() };
        let (tx, rx) = mpsc::channel::<NodeEvent>(30);
        println!("Node starting on {}", listen_addr);

        let connected_peers = Arc::new(Mutex::new(HashMap::new()));
        {
            let connected_peers = connected_peers.clone();
            tokio::spawn(async move { Self::start_node(connected_peers, rx).await });
        }
        tokio::spawn(crate::node::listener::start_listener(
            local_peer.clone(),
            listen_addr.to_string(),
            tx.clone(),
        ));
        println!("Node running on {}", listen_addr);

        Self {
            sender: tx,
            connected_peers,
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

    async fn start_node(peers: ConnectedPeers, mut rx: mpsc::Receiver<NodeEvent>) {
        while let Some(event) = rx.recv().await {
            match event {
                NodeEvent::PeerConnected(connection) => {
                    let mut pending = peers
                        .lock()
                        .expect("Unrecoverable failure: pending peers mutext poisoned");
                    pending.insert(connection.get_id(), connection);
                }
                NodeEvent::PeerDisconnected(id) => {
                    println!("Peer disconnected: {}", id);
                }
                NodeEvent::NetworkMessage(NetworkMessage {
                    peer_id,
                    protocol_id,
                    message,
                }) => {
                    println!("Message from {}:{:?}: {:?}", peer_id, protocol_id, message);
                }
            }
        }
    }
}
