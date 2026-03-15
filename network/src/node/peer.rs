use crate::comm::events::PeerConnectionEvent;
use crate::comm::net_message::NetworkMessage;
use crate::NetworkError;
use crate::{comm::events::NodeEvent, node::connection::UnverifiedConnection};
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
    match TcpStream::connect(addr.clone()).await {
        Ok(stream) => {
            tokio::spawn({
                let (read_half, write_half) = stream.into_split();
                let _ = node_tx
                    .send(NodeEvent::PeerConnection(
                        PeerConnectionEvent::IntializingConnection {
                            inbound: UnverifiedConnection::new(addr.clone(), write_half),
                            local_peer,
                        },
                    ))
                    .await;

                // TODO: move it to event handling
                /*
                   let peer = peer_handshake::initiate_protocol(local_peer, node_tx.clone())
                        .await
                        .unwrap();
                */

                handle_peer(read_half, node_tx, addr.clone())
            });
            Ok(())
        }
        Err(e) => Err(NetworkError::PeerFailure(
            format!("Failed to connect: {:?}", e).to_string(),
        )),
    }
}

pub async fn handle_peer(
    mut read_half: tokio::net::tcp::OwnedReadHalf,
    node_tx: mpsc::Sender<NodeEvent>,
    addr: String,
) -> io::Result<()> {
    /*
        node_tx
            .send(NodeEvent::PeerConnected(peer.clone()))
            .await
            .ok();
    */

    while let Ok(len) = read_half.read_u32().await {
        if len > 10_000 {
            break;
        }

        let mut buffer = vec![0u8; len as usize];
        read_half.read_exact(&mut buffer).await?;

        let message: NetworkMessage = bincode::deserialize(&buffer).unwrap();
        // here we would need to send an optional oneshot channel that we await on
        node_tx
            .send(NodeEvent::Message {
                peer_id: addr.clone(),
                message,
            })
            .await
            .ok();
    }

    node_tx.send(NodeEvent::PeerDisconnected(addr)).await.ok();
    Ok(())
}
