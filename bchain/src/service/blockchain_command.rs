use tokio::sync::mpsc;

pub enum BlockchainCommand {}

#[derive(Clone)]
pub struct CommandHandle {
    pub(super) tx: mpsc::Sender<BlockchainCommand>,
}
