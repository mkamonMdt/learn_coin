use clap::Parser;
use client::client_control::CtrlCommand;
use client::run_clinet;
use tokio::sync::mpsc;

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

    let (tx, rx) = mpsc::channel::<CtrlCommand>(10);
    tokio::spawn(async move {
        run_clinet(listen_addr, rx).await;
    });

    if let Some(peer_addr) = args.connect_to {
        let _ = tx.send(CtrlCommand::InitiateConnection(peer_addr)).await;
    }
}
