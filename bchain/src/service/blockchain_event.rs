use tokio::sync::broadcast;

pub enum BlockchainEvent {}

pub struct EventSubscribtion {
    pub(super) rx: broadcast::Receiver<BlockchainEvent>,
}
