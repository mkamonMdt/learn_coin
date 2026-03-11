use std::io;
use tokio::{io::AsyncReadExt, net::TcpStream, sync::mpsc};
use uuid::Uuid;

use crate::comm::events::NodeEvent;
use crate::comm::net_message::NetworkMessage;
use crate::protocols::peer_handshake::run_protocol;
use crate::NetworkError;

#[derive(Debug, Clone, PartialEq)]
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
) -> Result<(), NetworkError> {
    match TcpStream::connect(addr).await {
        Ok(stream) => {
            tokio::spawn(crate::node::peer::handle_peer(stream, node_tx));
            Ok(())
        }
        Err(e) => Err(NetworkError::PeerFailure(
            format!("Failed to connect: {:?}", e).to_string(),
        )),
    }
}

pub async fn handle_peer(stream: TcpStream, node_tx: mpsc::Sender<NodeEvent>) -> io::Result<()> {
    let (mut stream, peer) = run_protocol(stream, node_tx.clone()).await.unwrap();

    node_tx
        .send(NodeEvent::PeerConnected(peer.clone()))
        .await
        .ok();

    while let Ok(len) = stream.read_u32().await {
        if len > 10_000 {
            break;
        }

        let mut buffer = vec![0u8; len as usize];
        stream.read_exact(&mut buffer).await?;

        let message: NetworkMessage = bincode::deserialize(&buffer).unwrap();
        // here we would need to send an optional oneshot channel that we await on
        node_tx
            .send(NodeEvent::Message {
                peer_id: peer.id.to_string(),
                message,
            })
            .await
            .ok();
    }

    node_tx
        .send(NodeEvent::PeerDisconnected(peer.id.to_string()))
        .await
        .ok();
    Ok(())
}
