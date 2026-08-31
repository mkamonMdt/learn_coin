use tokio::sync::mpsc;
pub enum NetworkMessage {}

#[derive(Clone)]
pub struct NetworkMessageHandle {
    pub(super) tx: mpsc::Sender<NetworkMessage>,
}
