use std::fmt;

#[derive(Debug)]
pub enum FossilP2pError {
    Fossil(String),
    Identity(String),
    Storage(String),
    P2p(String),
    Config(String),
    Repository(String),
    Io(std::io::Error),
    Sqlite(rusqlite::Error),
    Serde(serde_json::Error),
    Crypto(String),
}

impl fmt::Display for FossilP2pError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Fossil(msg) => write!(f, "fossil error: {msg}"),
            Self::Identity(msg) => write!(f, "identity error: {msg}"),
            Self::Storage(msg) => write!(f, "storage error: {msg}"),
            Self::P2p(msg) => write!(f, "p2p error: {msg}"),
            Self::Config(msg) => write!(f, "config error: {msg}"),
            Self::Repository(msg) => write!(f, "repository error: {msg}"),
            Self::Io(err) => write!(f, "io error: {err}"),
            Self::Sqlite(err) => write!(f, "database error: {err}"),
            Self::Serde(err) => write!(f, "serialization error: {err}"),
            Self::Crypto(msg) => write!(f, "crypto error: {msg}"),
        }
    }
}

impl std::error::Error for FossilP2pError {}

impl From<std::io::Error> for FossilP2pError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

impl From<rusqlite::Error> for FossilP2pError {
    fn from(e: rusqlite::Error) -> Self {
        Self::Sqlite(e)
    }
}

impl From<serde_json::Error> for FossilP2pError {
    fn from(e: serde_json::Error) -> Self {
        Self::Serde(e)
    }
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
