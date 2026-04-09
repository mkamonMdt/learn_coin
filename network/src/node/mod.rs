pub mod listener;
pub mod peer;

use crate::comm::events::NodeEvent;
use crate::comm::p2p_connection::P2PConnection;
use crate::comm::p2p_connection::ProtocolHandle;
use crate::NetworkError;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;
use tokio::sync::mpsc;
use uuid::Uuid;

type ConnectedPeers = Arc<Mutex<HashMap<Uuid, P2PConnection>>>;

pub struct Node {
    event_sender: mpsc::Sender<NodeEvent>,
    connected_peers: ConnectedPeers,
}

impl Node {
    pub async fn new(listen_addr: String, event_sender: mpsc::Sender<NodeEvent>) -> Self {
        println!("Node starting on {}", listen_addr);

        let connected_peers = Arc::new(Mutex::new(HashMap::new()));
        tokio::spawn(crate::node::listener::start_listener(
            listen_addr.to_string(),
            connected_peers.clone(),
            event_sender.clone(),
        ));
        println!("Node running on {}", listen_addr);

        Self {
            event_sender,
            connected_peers,
        }
    }

    pub async fn bootstrap(&self, peer_addr: String) -> Option<Uuid> {
        println!("Connecting to {}", peer_addr);
        match peer::connect_to_peer(peer_addr.clone(), self.event_sender.clone()).await {
            Ok(connection) => {
                let mut pending = self
                    .connected_peers
                    .lock()
                    .expect("Unrecoverable failure: pending peers mutext poisoned");
                let id = connection.get_id();
                pending.insert(id, connection);
                return Some(id);
            }
            Err(e) => println!("Network error: {:#?}", e),
        }
        None
    }

    pub async fn open_protocol(
        &self,
        peer: Uuid,
        protocol_id: u16,
    ) -> Result<ProtocolHandle, NetworkError> {
        let handle = {
            let connected_peers = self
                .connected_peers
                .lock()
                .expect("Unrecoverable failure: pending peers mutext poisoned");
            let connection = connected_peers.get(&peer).unwrap();
            connection.get_uninit_handle()
        };

        Ok(handle.open_protocol(protocol_id).await)
    }
}
