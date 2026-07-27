use std::io::{self, BufRead};

use tokio::sync::mpsc;

use crate::client_control::CtrlCommand;

pub fn run_cli(client_ctrl_tx: mpsc::Sender<CtrlCommand>) {
    let stdin = io::stdin();
    let mut lines = stdin.lock().lines();

    loop {
        print!(">");
        io::Write::flush(&mut io::stdout()).unwrap();

        let line = match lines.next() {
            Some(Ok(l)) => l.trim().to_string(),
            _ => break,
        };

        if line.is_empty() {
            continue;
        }

        match line.split_whitespace().collect::<Vec<&str>>().as_slice() {
            ["exit"] | ["quit"] => {
                println!("👋 Goodbye!");
                break;
            }
            ["hello"] => println!("Hello there! 👋"),
            ["connect", port] => {
                if let Ok(port) = port.parse::<u32>() {
                    let peer_addr = format!("127.0.0.1:{}", port);
                    let tx = client_ctrl_tx.clone();
                    tokio::spawn(async move {
                        let _ = tx.send(CtrlCommand::InitiateConnection(peer_addr)).await;
                    });
                }
            }
            x => println!("Here goes nothing {:?}", x),
        }
    }
}
