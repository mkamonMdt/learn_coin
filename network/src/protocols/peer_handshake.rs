use crate::comm::events::NodeEvent;
use crate::comm::events::PeerConnectionEvent;
use crate::node::peer::Peer;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tokio::sync::mpsc;
use tokio::sync::oneshot;

pub async fn initiate_protocol(
    local_peer: Peer,
    event_handler: mpsc::Sender<NodeEvent>,
) -> Result<Peer, ()> {
    let local_iv = generate_rand_iv();
    let msg = PeerHandshake::new(local_peer.clone(), local_iv);
    let (tx, rx) = oneshot::channel();
    let _ = event_handler
        .send(NodeEvent::PeerConnection(
            PeerConnectionEvent::PeerHandshake(msg, tx),
        ))
        .await;

    let (remote_peer, remote_iv) = match rx.await.unwrap() {
        PeerHandshake::Response(proof) => {
            if !proof.verify_incomming(&local_peer, local_iv) {
                return Err(());
            }
            (proof.sender, proof.sender_iv)
        }
        _ => {
            return Err(());
        }
    };

    let req = ProofOfPossessionRequest {
        iv: remote_iv,
        peer: remote_peer.clone(),
    };
    let msg = PeerHandshake::from_request(req, local_peer, local_iv);
    let (tx, _rx) = oneshot::channel();
    let _ = event_handler
        .send(NodeEvent::PeerConnection(
            PeerConnectionEvent::PeerHandshake(msg, tx),
        ))
        .await;

    Ok(remote_peer)
}

pub async fn accept_protocol(_event_handler: mpsc::Sender<NodeEvent>) -> Result<Peer, ()> {
    todo!()
}

#[derive(Serialize, Deserialize, Debug)]
pub enum PeerHandshake {
    Request(ProofOfPossessionRequest),
    Response(ProofOfPossessionResponse),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ProofOfPossessionRequest {
    /// TODO: should be a type, preferebly an enum to update versions
    iv: [u8; 8],
    peer: Peer,
}

#[derive(Serialize, Deserialize, Debug)]
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
    fn from_request(req: ProofOfPossessionRequest, local_peer: Peer, local_iv: [u8; 8]) -> Self {
        let mut hasher = Sha256::new();

        hasher.update(req.peer.to_bytes());
        hasher.update(req.iv);
        hasher.update(local_peer.to_bytes());
        hasher.update(local_iv);

        Self::Response(ProofOfPossessionResponse {
            sender: local_peer,
            receiver: req.peer,
            sender_iv: local_iv,
            receiver_iv: req.iv,
            signature: hasher.finalize().into(),
        })
    }

    fn new(peer: Peer, iv: [u8; 8]) -> Self {
        Self::Request(ProofOfPossessionRequest { iv, peer })
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
