mod protocols;

use crate::protocols::TwoPartyExchange;
use crate::protocols::peer_handshake::HandshakeProtocol;
use network::comm::events::AlfaProtocols;
use network::comm::events::NodeEvent;
use network::comm::events::ProtocolId;
use network::node::Node;
use network::node::peer::Peer;
use std::sync::Arc;
use tokio::sync::mpsc;
use uuid::Uuid;

pub async fn run_clinet(local_addr: String, bootstrap: Option<String>) {
    let (tx, mut rx) = mpsc::channel::<NodeEvent>(10);

    let node = Arc::new(Node::new(local_addr, tx).await);
    let local_peer = Peer { id: Uuid::new_v4() };
    if let Some(peer_addr) = bootstrap
        && let Some(peer) = node.bootstrap(peer_addr).await
    {
        let local_peer = local_peer.clone();
        let protocol_handle = node
            .open_protocol(peer, ProtocolId::V0(AlfaProtocols::Handshake))
            .await
            .unwrap();

        tokio::spawn(async move {
            HandshakeProtocol::from(local_peer)
                .initiate(protocol_handle)
                .await;
        });
    }

    println!("{:?} ---client--- running", local_peer);

    loop {
        tokio::select! {
            Some(event) = rx.recv()=>
            {

                let local_peer = local_peer.clone();
                let node = node.clone();
                tokio::spawn(async move{
                    handle_network_event(local_peer ,node, event).await;
                });
            }


        }
    }
}

async fn handle_network_event(local_peer: Peer, node: Arc<Node>, event: NodeEvent) {
    match event {
        NodeEvent::PeerConnected(uuid) => {
            let protocol_handle = node
                .open_protocol(uuid, ProtocolId::V0(AlfaProtocols::Handshake))
                .await
                .unwrap();
            HandshakeProtocol::from(local_peer)
                .accept(protocol_handle)
                .await;
        }
        NodeEvent::PeerDisconnected(uuid) => println!("Peer disconnected {}", uuid),
        NodeEvent::NetworkMessage(_network_message) => todo!(),
    }
}
