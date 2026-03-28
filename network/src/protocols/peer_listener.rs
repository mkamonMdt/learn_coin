use crate::comm::events::NodeEvent;
use crate::comm::P2PReceiver;

use tokio::sync::mpsc;
use uuid::Uuid;

pub async fn listen_peer<P: P2PReceiver>(
    mut reader: P,
    node_tx: mpsc::Sender<NodeEvent>,
    peer_id: Uuid,
) -> std::io::Result<()> {
    while let Ok(msg) = reader.recieve().await {
        node_tx.send(NodeEvent::NetworkMessage(msg)).await.ok();
    }

    node_tx
        .send(NodeEvent::PeerDisconnected(peer_id))
        .await
        .ok();
    Ok(())
}
