use std::collections::HashMap;

use tokio::net::TcpStream;
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::comm::events::{NetworkMessage, ProtocolId};
use crate::comm::{P2PMessenger, P2PReceiver, P2PSender};

enum ProtocolCmd {
    Open(ProtocolId, mpsc::Sender<NetworkMessage>),
    Close(ProtocolId),
}

#[derive(Clone)]
pub struct UnintiProtocolHandle {
    outgoing_tx: mpsc::Sender<NetworkMessage>,
    register_tx: mpsc::Sender<ProtocolCmd>,
}

pub struct ProtocolHandle {
    outgoing_tx: mpsc::Sender<NetworkMessage>,
    incomming_rx: mpsc::Receiver<NetworkMessage>,
    register_tx: mpsc::Sender<ProtocolCmd>,
    protocol_id: ProtocolId,
}

impl P2PSender for ProtocolHandle {
    async fn send(&mut self, msg: NetworkMessage) -> std::io::Result<()> {
        let _ = self.outgoing_tx.send(msg).await;
        Ok(())
    }
}

impl P2PReceiver for ProtocolHandle {
    async fn recieve(&mut self) -> std::io::Result<NetworkMessage> {
        self.incomming_rx.recv().await.ok_or(std::io::Error::other(
            "ProtocolHandle: invalid receive event",
        ))
    }
}

impl P2PMessenger for ProtocolHandle {}

impl Drop for ProtocolHandle {
    fn drop(&mut self) {
        let _ = self
            .register_tx
            .try_send(ProtocolCmd::Close(self.protocol_id));
    }
}

pub struct P2PConnection {
    id: Uuid,
    outgoing_tx: mpsc::Sender<NetworkMessage>,
    register_tx: mpsc::Sender<ProtocolCmd>,
    _deamon_handle: tokio::task::JoinHandle<()>,
}

impl P2PConnection {
    pub async fn new(stream: TcpStream) -> Self {
        let (outgoing_tx, outgoing_rx) = mpsc::channel::<NetworkMessage>(10);
        let (register_tx, register_rx) = mpsc::channel::<ProtocolCmd>(10);

        let handle = tokio::spawn(backend_deamon(stream, outgoing_rx, register_rx));

        Self {
            id: Uuid::new_v4(),
            outgoing_tx,
            register_tx,
            _deamon_handle: handle,
        }
    }

    pub fn get_id(&self) -> Uuid {
        self.id
    }

    pub fn get_uninit_handle(&self) -> UnintiProtocolHandle {
        UnintiProtocolHandle {
            outgoing_tx: self.outgoing_tx.clone(),
            register_tx: self.register_tx.clone(),
        }
    }
}

impl UnintiProtocolHandle {
    pub async fn open_protocol(self, protocol_id: ProtocolId) -> ProtocolHandle {
        let (incomming_tx, incomming_rx) = mpsc::channel::<NetworkMessage>(10);

        //TODO: Error handling
        let _ = self
            .register_tx
            .send(ProtocolCmd::Open(protocol_id, incomming_tx.clone()))
            .await;

        ProtocolHandle {
            outgoing_tx: self.outgoing_tx,
            incomming_rx,
            register_tx: self.register_tx,
            protocol_id,
        }
    }
}

async fn backend_deamon(
    stream: TcpStream,
    mut outgoing_rx: mpsc::Receiver<NetworkMessage>,
    mut register_rx: mpsc::Receiver<ProtocolCmd>,
) {
    let mut protocol_registry: HashMap<ProtocolId, mpsc::Sender<NetworkMessage>> = HashMap::new();
    let (mut reader, mut writer) = stream.into_split();
    loop {
        tokio::select! {
            Some(payload) = outgoing_rx.recv() => {
                let _ = writer.send(payload).await;
            }
            Ok(incomming) = reader.recieve()=> {
                if let Some(tx) = protocol_registry.get(&incomming.protocol_id) {
                    let _ = tx.send(incomming).await;
                }
            }
            Some(cmd) = register_rx.recv() => {
                match cmd {
                    ProtocolCmd::Open(protocol_id, sender) => {protocol_registry.insert(protocol_id, sender);
                    }
                    ProtocolCmd::Close(protocol_id) =>{ protocol_registry.remove(&protocol_id);}
                }

            }
        }
    }
}
