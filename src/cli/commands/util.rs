use crate::config::FossilP2pConfig;
use crate::crypto::Keypair;
use crate::error::Result;
use crate::home::Home;
use crate::storage::Database;

pub fn get_home() -> Result<Home> {
    Home::new()
}

pub fn get_db(home: &Home) -> Result<Database> {
    Database::open(&home.db())
}

pub fn get_keypair(home: &Home) -> Result<Keypair> {
    crate::keystore::load_or_create_keypair(home)
}

pub fn get_config(home: &Home) -> Result<FossilP2pConfig> {
    FossilP2pConfig::load(&home.config())
}

pub fn default_repo_name(path: &std::path::Path) -> String {
    path.file_name()
        .and_then(|n| n.to_str())
        .map(|s| s.to_string())
        .unwrap_or_else(|| "my-project".to_string())
}