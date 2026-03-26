use crate::comm::read_msg;
use crate::NetworkError;
use crate::{comm::events::NodeEvent, protocols::peer_handshake::initiate_protocol};
use serde::{Deserialize, Serialize};
use tokio::{net::TcpStream, sync::mpsc};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Peer {
    pub addr: String,
    pub id: Uuid,
}

impl Peer {
    // TODO: to verify in future: could we have no allocation (const array size)?
    // it seems so at the moment.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = vec![];

        bytes.extend(self.addr.bytes());
        bytes.extend_from_slice(self.id.as_bytes());
        bytes
    }
}

pub async fn connect_to_peer(
    addr: String,
    node_tx: mpsc::Sender<NodeEvent>,
    local_peer: Peer,
) -> Result<(), NetworkError> {
    match TcpStream::connect(addr.clone()).await {
        Ok(stream) => {
            tokio::spawn({
                let (peer, reader, writer) = initiate_protocol(local_peer, stream).await.unwrap();
                let _ = node_tx.send(NodeEvent::PeerConnected(peer, writer)).await;
                handle_peer(reader, node_tx, addr.clone())
            });
            Ok(())
        }
        Err(e) => Err(NetworkError::PeerFailure(
            format!("Failed to connect: {:?}", e).to_string(),
        )),
    }
}

pub async fn handle_peer(
    mut reader: tokio::net::tcp::OwnedReadHalf,
    node_tx: mpsc::Sender<NodeEvent>,
    addr: String,
) -> std::io::Result<()> {
    while let Ok(msg) = read_msg(&mut reader).await {
        node_tx.send(NodeEvent::NetworkMessage(msg)).await.ok();
    }

    node_tx.send(NodeEvent::PeerDisconnected(addr)).await.ok();
    Ok(())
}
