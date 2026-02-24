use std::env;
use tokio::sync::mpsc;

mod comm;
mod node;

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    let listen_addr = args.get(1).cloned().unwrap_or("127.0.0.1:7000".to_string());
    let bootstrap = args.get(2).cloned(); // optional peer

    let (node_tx, node_rx) = mpsc::channel(100);

    // Start Node Core
    tokio::spawn(node::start_node(node_rx));

    // Start Lisitner
    tokio::spawn(node::listener::start_listener(
        listen_addr.clone(),
        node_tx.clone(),
    ));

    println!("Node running on {}", listen_addr);

    // If bootstrap provided → connect
    if let Some(peer_addr) = bootstrap {
        println!("Connecting to {}", peer_addr);
        node::peer::connect_to_peer(peer_addr, node_tx.clone()).await;
    }

    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
    }
}
