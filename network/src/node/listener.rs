use crate::comm::events::NodeEvent;
use crate::node::peer::handle_peer;
use tokio::{net::TcpListener, sync::mpsc};

pub async fn start_listener(addr: String, node_tx: mpsc::Sender<NodeEvent>) {
    let listener = TcpListener::bind(addr).await.expect("Failed to bind");

    loop {
        let (socket, _) = listener.accept().await.unwrap();
        tokio::spawn(handle_peer(socket, node_tx.clone()));
    }
}
