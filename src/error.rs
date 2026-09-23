use thiserror::Error;

#[derive(Error, Debug)]
pub enum FossilP2pError {
    #[error("fossil error: {0}")]
    Fossil(String),
    #[error("identity error: {0}")]
    Identity(String),
    #[error("storage error: {0}")]
    Storage(String),
    #[error("p2p error: {0}")]
    P2p(String),
    #[error("config error: {0}")]
    Config(String),
    #[error("repository error: {0}")]
    Repository(String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("database error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("serialization error: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("crypto error: {0}")]
    Crypto(String),
    #[error("protocol error: {0}")]
    Protocol(String),
    #[error("authorization error: {0}")]
    Authorization(String),
    #[error("network error: {0}")]
    Network(String),
}

impl From<String> for FossilP2pError {
    fn from(e: String) -> Self {
        Self::P2p(e)
    }
}

impl From<&str> for FossilP2pError {
    fn from(e: &str) -> Self {
        Self::P2p(e.to_string())
    }
}

pub type Result<T> = std::result::Result<T, FossilP2pError>;