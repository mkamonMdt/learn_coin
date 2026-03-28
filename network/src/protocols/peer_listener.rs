use crate::comm::events::{NetworkMessage, NodeEvent};
use crate::comm::PeerReader;

use tokio::sync::mpsc;
use uuid::Uuid;

pub async fn listen_peer(
    mut reader: impl PeerReader<NetworkMessage>,
    node_tx: mpsc::Sender<NodeEvent>,
    peer_id: Uuid,
) -> std::io::Result<()> {
    while let Ok(msg) = reader.read_from_peer().await {
        node_tx.send(NodeEvent::NetworkMessage(msg)).await.ok();
    }

    node_tx
        .send(NodeEvent::PeerDisconnected(peer_id))
        .await
        .ok();
    Ok(())
}
