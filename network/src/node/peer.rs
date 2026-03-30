use crate::comm::p2p_connection::P2PConnection;
use crate::NetworkError;
use serde::{Deserialize, Serialize};
use tokio::net::TcpStream;
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

pub async fn connect_to_peer(addr: String) -> Result<P2PConnection, NetworkError> {
    match TcpStream::connect(addr.clone()).await {
        Ok(stream) => {
            let connection = P2PConnection::new(stream).await;
            Ok(connection)
        }
        Err(e) => Err(NetworkError::PeerFailure(
            format!("Failed to connect: {:?}", e).to_string(),
        )),
    }
}
