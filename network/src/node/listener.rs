use crate::comm::events::NodeEvent;
use crate::node::peer::handle_peer;
use tokio::{net::TcpListener, sync::mpsc};

pub async fn start_listener(addr: String, node_tx: mpsc::Sender<NodeEvent>) -> ! {
    let listener = TcpListener::bind(addr.clone())
        .await
        .expect("Failed to bind");

    loop {
        //
        let (stream, _) = listener.accept().await.unwrap();
        let (read_half, _write_half) = stream.into_split();
        let node_tx = node_tx.clone();
        let addr = addr.clone();
        tokio::spawn(async move {
            // TODO: move it to event handling
            // let peer = accept_protocol(node_tx.clone()).await.unwrap();

            // TODO: self writing? Do I need that?
            /*
                        let _ = node_tx
                            .send(NodeEvent::PeerConnection(
                                PeerConnectionEvent::IncommingConnection(UnverifiedConnection::new(
                                    addr.clone(),
                                    write_half,
                                )),
                            ))
                            .await;
            */
            let _ = handle_peer(read_half, node_tx, addr).await;
        });
    }
}
