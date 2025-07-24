use thiserror::Error;

#[derive(Error, Debug)]
pub enum BChainError {
    #[error("DummyError: {0}")]
    DummyErrur(String),
    #[error("User: {0} not found")]
    UserNotFound(String),
    #[error("User: {0} failed to produce bloch with err:{1}")]
    BlockProductionFailure(String, String),
}
