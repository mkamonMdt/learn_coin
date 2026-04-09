use crate::protocols::{ProtocolId, TwoPartyExchange};
use network::NetworkError;
use network::comm::P2PMessenger;
use network::comm::events::NetworkMessage;
use network::node::peer::Peer;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

pub struct HandshakeProtocol {
    local_peer: Peer,
}

impl ProtocolId for HandshakeProtocol {
    const PREFIX: u8 = 1;

    /// Only single HandshakeProtocol per peer supported
    fn value(&self) -> u8 {
        1
    }
}

impl From<Peer> for HandshakeProtocol {
    fn from(local_peer: Peer) -> Self {
        Self { local_peer }
    }
}

impl TwoPartyExchange for HandshakeProtocol {
    async fn initiate(self, mut messanger: impl P2PMessenger) {
        println!(
            "{:?}: ---init---- initiating handshake protocol",
            self.local_peer
        );
        let local_iv = generate_rand_iv();
        let msg = PeerHandshake::Request(ProofOfPossessionRequest {
            iv: local_iv,
            peer: self.local_peer.clone(),
        });

        let response = PeerHandshake::try_from(
            messanger
                .send_receive(msg.try_into(self.to_u16()).unwrap())
                .await
                .unwrap(),
        )
        .unwrap();
        println!(
            "{:?}: ---init---- handshake 1st response received",
            self.local_peer
        );

        let (remote_peer, remote_iv) = match response {
            PeerHandshake::Response(proof) => {
                if !proof.verify_incomming(&self.local_peer, local_iv) {
                    println!(
                        "{:?}: ---init---- Handshake: verification failure",
                        self.local_peer
                    );
                    return;
                }
                (proof.sender, proof.sender_iv)
            }
            _ => {
                println!(
                    "{:?}: ---init---- Handshake: invalid state",
                    self.local_peer
                );
                return;
            }
        };
        println!(
            "{:?}: ---init---- handshake 1st response verified from {:?}",
            self.local_peer, remote_peer
        );

        let req = ProofOfPossessionRequest {
            iv: remote_iv,
            peer: remote_peer.clone(),
        };
        let msg = PeerHandshake::from_request(req, self.local_peer.clone(), local_iv);
        let _ = messanger.send(msg.try_into(self.to_u16()).unwrap()).await;
        println!(
            "{:?}: ---init---- handshake 2nd response sent",
            self.local_peer
        );
    }

    async fn accept(self, mut messanger: impl P2PMessenger) {
        // read request
        let request = PeerHandshake::try_from(messanger.recieve().await.unwrap()).unwrap();
        let local_iv = generate_rand_iv();
        println!(
            "{:?}: ---acc---- handshake request received",
            self.local_peer
        );

        // send response
        let response = match request {
            PeerHandshake::Request(req) => {
                let response = PeerHandshake::from_request(req, self.local_peer.clone(), local_iv);
                PeerHandshake::try_from(
                    messanger
                        .send_receive(response.try_into(self.to_u16()).unwrap())
                        .await
                        .unwrap(),
                )
                .unwrap()
            }
            _ => {
                println!("{:?}: ---acc--- Handshake: invalid state ", self.local_peer);
                return;
            }
        };
        println!(
            "{:?}: ---acc---- handshake response received",
            self.local_peer
        );
        match response {
            PeerHandshake::Response(proof) => {
                if !proof.verify_incomming(&self.local_peer, local_iv) {
                    println!(
                        "{:?}: ---acc--- Handshake: verification failure",
                        self.local_peer
                    );
                    return;
                }
            }
            _ => {
                println!("{:?}: ---acc--- Handshake: invalid state ", self.local_peer);
                return;
            }
        };
        println!(
            "{:?}: ---acc---- handshake 2nd response verified",
            self.local_peer
        );
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub enum PeerHandshake {
    Request(ProofOfPossessionRequest),
    Response(ProofOfPossessionResponse),
}

impl PeerHandshake {
    fn try_into(self, protocol_id: u16) -> Result<NetworkMessage, NetworkError> {
        //TODO: solve id problem
        let peer_id = Uuid::new_v4();
        let message = bincode::serialize(&self).map_err(|e| {
            NetworkError::PeerFailure(format!("Handshake: serialization failure:{}", e))
        })?;

        Ok(NetworkMessage {
            peer_id,
            protocol_id,
            message,
        })
    }

    fn try_from(value: NetworkMessage) -> Result<Self, NetworkError> {
        bincode::deserialize(&value.message).map_err(|e| {
            NetworkError::PeerFailure(format!("Handshake: Could not deserialize:{}", e))
        })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
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

        hasher.update(local_peer.to_bytes());
        hasher.update(local_iv);
        hasher.update(req.peer.to_bytes());
        hasher.update(req.iv);

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
