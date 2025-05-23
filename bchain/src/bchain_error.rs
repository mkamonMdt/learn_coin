use thiserror::Error;

#[derive(Error, Debug)]
pub enum BChainError {
    #[error("DummyError: {0}")]
    DummyErrur(String),
}
