use thiserror::Error;

#[derive(Error, Debug)]
pub enum NetworkError {
    #[error("Peer failure: {0}")]
    PeerFailure(String),
}
