mod blockchain_command;
mod blockchain_event;
mod network_message;

use crate::blockchain::Blockchain;
use crate::service::blockchain_event::EventSubscribtion;
use tokio::sync::broadcast;
use tokio::sync::mpsc;

pub use blockchain_command::BlockchainCommand;
pub use blockchain_command::CommandHandle;
pub use blockchain_event::BlockchainEvent;
pub use network_message::NetworkMessage;
pub use network_message::NetworkMessageHandle;

pub struct BlockchainService {
    state: Blockchain,

    cmd_rx: mpsc::Receiver<BlockchainCommand>,
    network_rx: mpsc::Receiver<NetworkMessage>,
    event_tx: broadcast::Sender<BlockchainEvent>,
}

impl BlockchainService {
    fn start(blockchain: Blockchain) -> ServiceHandles {
        // check if blockchain is at genesis
        let (cmd_tx, cmd_rx) = mpsc::channel(16);
        let (network_tx, network_rx) = mpsc::channel(128);
        let (event_tx, _) = broadcast::channel(128);

        let mut x = Self {
            state: blockchain,
            cmd_rx,
            network_rx,
            event_tx: event_tx.clone(),
        };

        tokio::spawn(async move {
            x.start_listener().await;
        });

        ServiceHandles {
            cmd_tx,
            network_tx,
            event_tx,
        }
    }

    async fn start_listener(&mut self) {
        loop {
            tokio::select! {
                Some(_cmd) = self.cmd_rx.recv() => {
                    todo!()
                }
                Some(_msg) = self.network_rx.recv() => {
                    todo!()
                }
            }
        }
    }
}

#[derive(Clone)]
pub struct ServiceHandles {
    cmd_tx: mpsc::Sender<BlockchainCommand>,
    network_tx: mpsc::Sender<NetworkMessage>,
    event_tx: broadcast::Sender<BlockchainEvent>,
}

impl ServiceHandles {
    fn get_cmd_handle(&self) -> CommandHandle {
        CommandHandle {
            tx: self.cmd_tx.clone(),
        }
    }
    fn get_network_handle(&self) -> NetworkMessageHandle {
        NetworkMessageHandle {
            tx: self.network_tx.clone(),
        }
    }

    fn event_subscribe(&self) -> EventSubscribtion {
        EventSubscribtion {
            rx: self.event_tx.subscribe(),
        }
    }
}
