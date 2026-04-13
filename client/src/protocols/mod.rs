use network::comm::P2PMessenger;
use network::NetworkError;
use thiserror::Error;

pub mod peer_handshake;

#[derive(Error, Debug)]
pub enum ProtocolError {
    #[error("Peer failed with {0}")]
    Network(#[from] NetworkError),

    #[error("Protocol {protocol} failed with {msg}")]
    ProtocolStep { protocol: String, msg: String },

    #[error("Message encoding failed: {0}")]
    Encoding(String),

    #[error("Message decoding failed: {0}")]
    Decoding(String),
}

pub trait TwoPartyExchange {
    async fn initiate(self, messanger: impl P2PMessenger) -> Result<(), ProtocolError>;
    async fn accept(self, messanger: impl P2PMessenger) -> Result<(), ProtocolError>;
}

pub trait ProtocolId {
    /// Must be unique across all protocols
    const PREFIX: u8;

    /// For future implementation of handling multiple protocols
    /// of the same type in parallel
    fn value(&self) -> u8;

    fn to_u16(&self) -> u16 {
        ((Self::PREFIX as u16) << 8) | (self.value() as u16)
    }
}
