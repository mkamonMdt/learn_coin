use crate::comm::events::NodeEvent;
use crate::comm::net_message::NetworkMessage;
use crate::protocols::peer_handshake;
use crate::NetworkError;
use serde::{Deserialize, Serialize};
use std::io;
use tokio::{io::AsyncReadExt, net::TcpStream, sync::mpsc};
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
    match TcpStream::connect(addr).await {
        Ok(stream) => {
            tokio::spawn({
                // NOTE: It will not work yet, there is no proper send/receive
                // data from TcpStream implemented. We need to create TcpStream blindly
                // and based on Handshake protocol result create new peer or
                // close the stream.
                let peer = peer_handshake::initiate_protocol(local_peer, node_tx.clone())
                    .await
                    .unwrap();
                crate::node::peer::handle_peer(stream, node_tx, peer)
            });
            Ok(())
        }
        Err(e) => Err(NetworkError::PeerFailure(
            format!("Failed to connect: {:?}", e).to_string(),
        )),
    }
}

pub async fn handle_peer(
    mut stream: TcpStream,
    node_tx: mpsc::Sender<NodeEvent>,
    peer: Peer,
) -> io::Result<()> {
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
