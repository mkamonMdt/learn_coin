use network::comm::P2PMessenger;

pub mod peer_handshake;

pub trait TwoPartyExchange {
    async fn initiate(self, messanger: impl P2PMessenger);
    async fn accept(self, messanger: impl P2PMessenger);
}
