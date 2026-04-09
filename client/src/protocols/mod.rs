use network::comm::P2PMessenger;

pub mod peer_handshake;

pub trait TwoPartyExchange {
    async fn initiate(self, messanger: impl P2PMessenger);
    async fn accept(self, messanger: impl P2PMessenger);
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
