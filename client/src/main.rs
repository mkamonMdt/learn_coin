use clap::Parser;
use client::run_clinet;

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

    run_clinet(listen_addr, args.connect_to).await;

    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
    }
}
