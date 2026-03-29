use crate::comm::events::ProtocolId;
use crate::comm::p2p_connection::P2PConnection;
use crate::NetworkError;
use crate::{comm::events::NodeEvent, protocols::peer_handshake::initiate_protocol};
use serde::{Deserialize, Serialize};
use tokio::{net::TcpStream, sync::mpsc};
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
    node_tx: mpsc::Sender<NodeEvent>,
    local_peer: Peer,
) -> Result<(), NetworkError> {
    match TcpStream::connect(addr.clone()).await {
        Ok(stream) => {
            tokio::spawn(async move {
                let conneciton = P2PConnection::new(stream).await;
                let protocol_id = ProtocolId::V0(crate::comm::events::AlfaProtocols::Handshake);
                let handle = conneciton.open_protocol(protocol_id).await;
                let _ = node_tx.send(NodeEvent::PeerConnected(conneciton)).await;

                initiate_protocol(local_peer, handle).await;
            });
            Ok(())
        }
        Err(e) => Err(NetworkError::PeerFailure(
            format!("Failed to connect: {:?}", e).to_string(),
        )),
    }
}
