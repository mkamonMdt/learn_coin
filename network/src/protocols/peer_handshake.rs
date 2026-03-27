use crate::comm::{PeerReader, PeerWriter};
use crate::node::peer::Peer;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tokio::net::tcp::OwnedReadHalf;
use tokio::net::tcp::OwnedWriteHalf;
use tokio::net::TcpStream;

pub async fn initiate_protocol<R, W>(
    local_peer: Peer,
    mut reader: R,
    mut writer: W,
) -> Result<(Peer, R, W), ()>
where
    R: PeerReader<PeerHandshake>,
    W: PeerWriter<PeerHandshake>,
{
    println!(
        "{:?}: ---init---- initiating handshake protocol",
        local_peer
    );
    let local_iv = generate_rand_iv();
    let msg = PeerHandshake::Request(ProofOfPossessionRequest {
        iv: local_iv,
        peer: local_peer.clone(),
    });

    // let (mut reader, mut writer) = stream.into_split();

    let _ = writer.write_to_peer(msg).await;
    println!("{:?}: ---init---- handshake request sent", local_peer);

    let response: PeerHandshake = reader.read_from_peer().await.unwrap();
    println!(
        "{:?}: ---init---- handshake 1st response received",
        local_peer
    );
    let (remote_peer, remote_iv) = match response {
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
    println!(
        "{:?}: ---init---- handshake 1st response verified from {:?}",
        local_peer, remote_peer
    );

    let req = ProofOfPossessionRequest {
        iv: remote_iv,
        peer: remote_peer.clone(),
    };
    let msg = PeerHandshake::from_request(req, local_peer.clone(), local_iv);
    let _ = writer.write_to_peer(msg).await;
    println!("{:?}: ---init---- handshake 2nd response sent", local_peer);

    Ok((remote_peer, reader, writer))
}

pub async fn accept_protocol(
    local_peer: Peer,
    stream: TcpStream,
) -> Result<(Peer, OwnedReadHalf, OwnedWriteHalf), ()> {
    let (mut reader, mut writer) = stream.into_split();
    // read request
    let request: PeerHandshake = reader.read_from_peer().await.unwrap();
    let local_iv = generate_rand_iv();
    println!("{:?}: ---acc---- handshake request received", local_peer);

    // send response
    let remote_peer = match request {
        PeerHandshake::Request(req) => {
            let response = PeerHandshake::from_request(req.clone(), local_peer.clone(), local_iv);
            let _ = writer.write_to_peer(response).await;
            req.peer
        }
        _ => {
            return Err(());
        }
    };
    println!("{:?}: ---acc---- handshake 1st response sent", local_peer);

    // read & verify response
    let response: PeerHandshake = reader.read_from_peer().await.unwrap();
    println!(
        "{:?}: ---acc---- handshake 2nd response received",
        local_peer
    );
    match response {
        PeerHandshake::Response(proof) => {
            if !proof.verify_incomming(&local_peer, local_iv) {
                return Err(());
            }
        }
        _ => {
            return Err(());
        }
    };
    println!(
        "{:?}: ---acc---- handshake 2nd response verified",
        local_peer
    );

    Ok((remote_peer, reader, writer))
}

#[derive(Serialize, Deserialize, Debug)]
pub enum PeerHandshake {
    Request(ProofOfPossessionRequest),
    Response(ProofOfPossessionResponse),
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
