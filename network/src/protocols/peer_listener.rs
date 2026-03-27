use crate::comm::events::{NetworkMessage, NodeEvent};
use crate::comm::PeerReader;

use tokio::sync::mpsc;

pub async fn listen_peer(
    mut reader: impl PeerReader<NetworkMessage>,
    node_tx: mpsc::Sender<NodeEvent>,
    addr: String,
) -> std::io::Result<()> {
    while let Ok(msg) = reader.read_from_peer().await {
        node_tx.send(NodeEvent::NetworkMessage(msg)).await.ok();
    }

    node_tx.send(NodeEvent::PeerDisconnected(addr)).await.ok();
    Ok(())
}
