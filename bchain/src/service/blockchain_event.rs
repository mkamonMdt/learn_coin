use tokio::sync::broadcast;

#[derive(Clone)]
pub enum BlockchainEvent {}

pub struct EventSubscribtion {
    pub(super) rx: broadcast::Receiver<BlockchainEvent>,
}
