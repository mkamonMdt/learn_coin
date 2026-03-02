use std::io;
use tokio::{io::AsyncReadExt, net::TcpStream, sync::mpsc};
use uuid::Uuid;

use crate::comm::events::NodeEvent;
use crate::comm::net_message::NetworkMessage;
use crate::NetworkError;

pub struct Peer {
    pub addr: String,
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

pub async fn handle_peer(
    mut stream: TcpStream,
    node_tx: mpsc::Sender<NodeEvent>,
) -> io::Result<()> {
    let peer_id = Uuid::new_v4().to_string();

    node_tx
        .send(NodeEvent::PeerConnected(peer_id.clone()))
        .await
        .ok();

    while let Ok(len) = stream.read_u32().await {
        if len > 10_000 {
            break;
        }

        let mut buffer = vec![0u8; len as usize];
        stream.read_exact(&mut buffer).await?;

        let message: NetworkMessage = bincode::deserialize(&buffer).unwrap();
        node_tx
            .send(NodeEvent::Message {
                peer_id: peer_id.clone(),
                message,
            })
            .await
            .ok();
    }

    node_tx
        .send(NodeEvent::PeerDisconnected(peer_id))
        .await
        .ok();
    Ok(())
}
