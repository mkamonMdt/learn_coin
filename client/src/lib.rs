pub mod cli;
pub mod client_control;
mod protocols;

use crate::client_control::CtrlCommand;
use crate::protocols::peer_handshake::HandshakeProtocol;
use crate::protocols::ProtocolId;
use crate::protocols::TwoPartyExchange;
use network::comm::events::NodeEvent;
use network::node::peer::Peer;
use network::node::Node;
use std::sync::Arc;
use tokio::sync::mpsc;
use uuid::Uuid;

#[derive(Clone)]
pub(crate) struct Client {
    node: Arc<Node>,
    local_peer: Peer,
}

impl Client {
    fn from(node: Node) -> Self {
        let local_peer = Peer { id: Uuid::new_v4() };
        let node = Arc::new(node);
        println!("{:?} ---client--- running", local_peer);
        Self { node, local_peer }
    }
}

pub async fn run_clinet(local_addr: String, mut ctrl_rx: mpsc::Receiver<CtrlCommand>) {
    let (tx, mut rx) = mpsc::channel::<NodeEvent>(10);
    let node = Node::new(local_addr, tx).await;
    let client = Client::from(node);

    loop {
        tokio::select! {
            Some(network_event) = rx.recv() => {
                let client = client.clone();
                tokio::spawn(async move{
                    handle_network_event(client, network_event).await;
                });
            }
            Some(ctrl_event) = ctrl_rx.recv() => {
                let client = client.clone();
                tokio::spawn(async move{
                    match ctrl_event {
                        CtrlCommand::InitiateConnection(addr) => connect_peer(client, addr).await,
                    }
                });
            }
        }
    }
}

async fn connect_peer(client: Client, peer_addr: String) {
    if let Some(peer) = client.node.bootstrap(peer_addr.clone()).await {
        let protocol = HandshakeProtocol::from(client.local_peer);
        let protocol_handle = client
            .node
            .open_protocol(peer, protocol.to_u16())
            .await
            .unwrap();

        tokio::spawn(async move {
            if let Err(e) = protocol.initiate(protocol_handle).await {
                println!("---init--- {e}")
            }
        });
    }
}

async fn handle_network_event(client: Client, event: NodeEvent) {
    match event {
        NodeEvent::PeerConnected(uuid) => {
            let protocol = HandshakeProtocol::from(client.local_peer);
            let protocol_handle = client
                .node
                .open_protocol(uuid, protocol.to_u16())
                .await
                .unwrap();
            if let Err(e) = protocol.accept(protocol_handle).await {
                println!("---acc--- {e}");
            }
        }
        NodeEvent::PeerDisconnected(uuid) => println!("Peer disconnected {}", uuid),
        NodeEvent::NetworkMessage(_network_message) => todo!(),
    }
}
