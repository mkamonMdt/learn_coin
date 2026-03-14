use crate::node::peer::handle_peer;
use crate::{comm::events::NodeEvent, protocols::peer_handshake::accept_protocol};
use tokio::{net::TcpListener, sync::mpsc};

pub async fn start_listener(addr: String, node_tx: mpsc::Sender<NodeEvent>) -> ! {
    let listener = TcpListener::bind(addr).await.expect("Failed to bind");

    loop {
        //
        let (socket, _) = listener.accept().await.unwrap();
        let node_tx = node_tx.clone();
        tokio::spawn(async move {
            // NOTE: It will not work yet, there is no proper send/receive
            // data from TcpStream implemented. We need to accept TcpStream blindly
            // and based on Handshake protocol result create new peer or
            // close the stream.
            let peer = accept_protocol(node_tx.clone()).await.unwrap();
            let _ = handle_peer(socket, node_tx, peer).await;
        });
    }
}
