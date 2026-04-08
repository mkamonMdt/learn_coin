use crate::comm::{events::NodeEvent, p2p_connection::P2PConnection};
use crate::NetworkError;
use serde::{Deserialize, Serialize};
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Peer {
    pub id: Uuid,
}

impl Peer {
    pub fn to_bytes(&self) -> Vec<u8> {
        self.id.into()
    }
}

pub async fn connect_to_peer(
    addr: String,
    event_tx: mpsc::Sender<NodeEvent>,
) -> Result<P2PConnection, NetworkError> {
    match TcpStream::connect(addr.clone()).await {
        Ok(stream) => {
            let connection = P2PConnection::new(stream, event_tx).await;
            Ok(connection)
        }
        Err(e) => Err(NetworkError::PeerFailure(
            format!("Failed to connect: {:?}", e).to_string(),
        )),
    }
}
