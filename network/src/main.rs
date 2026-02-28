use clap::Parser;

mod comm;
mod node;

#[derive(Parser, Debug)]
struct Args {
    #[arg(long, default_value = "40001")]
    listen_port: u16,

    #[arg(long)]
    connect_to: Option<String>,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let listen_addr = format!("127.0.0.1:{}", args.listen_port);

    let node = network::node::Node::new(listen_addr).await;

    // If bootstrap provided → connect
    if let Some(peer_addr) = args.connect_to {
        node.bootstrap(peer_addr).await;
    }

    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
    }
}
