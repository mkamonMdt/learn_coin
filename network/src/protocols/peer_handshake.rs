use crate::comm::events::NodeEvent;
use crate::node::peer::Peer;
use rand::RngCore;
use sha2::{Digest, Sha256};
use tokio::net::TcpStream;
use tokio::sync::mpsc;

/// TODO: The function should handle entire handshake protocol
pub async fn run_protocol(
    _stream: TcpStream,
    _event_handler: mpsc::Sender<NodeEvent>,
) -> Result<(TcpStream, Peer), ()> {
    todo!()
}

#[derive(Debug)]
pub enum PeerHandshake {
    Request(ProofOfPossessionRequest),
    Response(ProofOfPossessionResponse),
}

#[derive(Debug)]
pub struct ProofOfPossessionRequest {
    /// TODO: should be a type, preferebly an enum to update versions
    iv: [u8; 8],
    peer: Peer,
}

#[derive(Debug)]
pub struct ProofOfPossessionResponse {
    sender: Peer,
    receiver: Peer,
    sender_iv: [u8; 8],
    receiver_iv: [u8; 8],
    /// TODO: should be a type, preferebly an enum to update versions
    signature: [u8; 32],
}

impl PeerHandshake {
    /// TODO: there should be also some local signer here
    fn from_request(req: ProofOfPossessionRequest, local_peer: Peer) -> Self {
        let local_iv = generate_rand_iv();
        let signature = sign(&req, &local_peer, local_iv);

        Self::Response(ProofOfPossessionResponse {
            sender: local_peer,
            receiver: req.peer,
            sender_iv: local_iv,
            receiver_iv: req.iv,
            signature,
        })
    }

    fn new(local_peer: Peer) -> Self {
        Self::Request(ProofOfPossessionRequest {
            iv: generate_rand_iv(),
            peer: local_peer,
        })
    }
}

impl ProofOfPossessionResponse {
    fn verify_incomming(&self, local_peer: &Peer, local_iv: [u8; 8]) -> bool {
        if *local_peer != self.receiver || local_iv != self.receiver_iv {
            return false;
        }

        let mut hasher = Sha256::new();

        hasher.update(self.sender.to_bytes());
        hasher.update(self.sender_iv);
        hasher.update(self.receiver.to_bytes());
        hasher.update(self.receiver_iv);
        let calculated_sig: [u8; 32] = hasher.finalize().into();
        self.signature == calculated_sig
    }
}

fn generate_rand_iv() -> [u8; 8] {
    let mut bytes = [0u8; 8];
    rand::rngs::OsRng.fill_bytes(&mut bytes);
    bytes
}

fn sign(req: &ProofOfPossessionRequest, local_peer: &Peer, local_iv: [u8; 8]) -> [u8; 32] {
    let mut hasher = Sha256::new();

    hasher.update(req.peer.to_bytes());
    hasher.update(req.iv);
    hasher.update(local_peer.to_bytes());
    hasher.update(local_iv);
    hasher.finalize().into()
}
