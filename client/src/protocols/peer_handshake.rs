use network::comm::events::{NetworkMessage, ProtocolId};
use network::comm::P2PMessenger;
use network::node::peer::Peer;
use network::NetworkError;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

pub async fn initiate_protocol(local_peer: Peer, mut messanger: impl P2PMessenger) {
    println!(
        "{:?}: ---init---- initiating handshake protocol",
        local_peer
    );
    let local_iv = generate_rand_iv();
    let msg = PeerHandshake::Request(ProofOfPossessionRequest {
        iv: local_iv,
        peer: local_peer.clone(),
    });

    let response: PeerHandshake = messanger
        .send_receive(msg.try_into().unwrap())
        .await
        .unwrap()
        .try_into()
        .unwrap();
    println!(
        "{:?}: ---init---- handshake 1st response received",
        local_peer
    );

    let (remote_peer, remote_iv) = match response {
        PeerHandshake::Response(proof) => {
            if !proof.verify_incomming(&local_peer, local_iv) {
                println!(
                    "{:?}: ---init---- Handshake: verification failure",
                    local_peer
                );
                return;
            }
            (proof.sender, proof.sender_iv)
        }
        _ => {
            println!("{:?}: ---init---- Handshake: invalid state", local_peer);
            return;
        }
    };
    println!(
        "{:?}: ---init---- handshake 1st response verified from {:?}",
        local_peer, remote_peer
    );

    let req = ProofOfPossessionRequest {
        iv: remote_iv,
        peer: remote_peer.clone(),
    };
    let msg = PeerHandshake::from_request(req, local_peer.clone(), local_iv);
    let _ = messanger.send(msg.try_into().unwrap()).await;
    println!("{:?}: ---init---- handshake 2nd response sent", local_peer);
}

pub async fn accept_protocol(local_peer: Peer, mut messanger: impl P2PMessenger) {
    // read request
    let request: PeerHandshake = messanger.recieve().await.unwrap().try_into().unwrap();
    let local_iv = generate_rand_iv();
    println!("{:?}: ---acc---- handshake request received", local_peer);

    // send response
    let response: PeerHandshake = match request {
        PeerHandshake::Request(req) => {
            let response = PeerHandshake::from_request(req, local_peer.clone(), local_iv);
            messanger
                .send_receive(response.try_into().unwrap())
                .await
                .unwrap()
                .try_into()
                .unwrap()
        }
        _ => {
            println!("{:?}: ---acc--- Handshake: invalid state ", local_peer);
            return;
        }
    };
    println!("{:?}: ---acc---- handshake response received", local_peer);
    match response {
        PeerHandshake::Response(proof) => {
            if !proof.verify_incomming(&local_peer, local_iv) {
                println!(
                    "{:?}: ---acc--- Handshake: verification failure",
                    local_peer
                );
                return;
            }
        }
        _ => {
            println!("{:?}: ---acc--- Handshake: invalid state ", local_peer);
            return;
        }
    };
    println!(
        "{:?}: ---acc---- handshake 2nd response verified",
        local_peer
    );
}

#[derive(Serialize, Deserialize, Debug)]
pub enum PeerHandshake {
    Request(ProofOfPossessionRequest),
    Response(ProofOfPossessionResponse),
}

impl TryInto<NetworkMessage> for PeerHandshake {
    type Error = NetworkError;

    fn try_into(self) -> Result<NetworkMessage, Self::Error> {
        //TODO: solve id problem
        let peer_id = Uuid::new_v4();
        let message = bincode::serialize(&self).map_err(|e| {
            NetworkError::PeerFailure(format!("Handshake: serialization failure:{}", e))
        })?;

        Ok(NetworkMessage {
            peer_id,
            protocol_id: ProtocolId::V0(network::comm::events::AlfaProtocols::Handshake),
            message,
        })
    }
}

impl TryFrom<NetworkMessage> for PeerHandshake {
    type Error = NetworkError;

    fn try_from(value: NetworkMessage) -> Result<Self, Self::Error> {
        if let ProtocolId::V0(network::comm::events::AlfaProtocols::Handshake) = value.protocol_id {
            Ok(bincode::deserialize(&value.message).map_err(|e| {
                NetworkError::PeerFailure(format!("Handshake: Could not deserialize:{}", e))
            })?)
        } else {
            Err(NetworkError::PeerFailure(
                "Handshake: invalid msg received".to_string(),
            ))
        }
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
