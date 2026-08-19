use crate::error::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FossilP2pConfig {
    #[serde(default)]
    pub node: NodeConfig,
    #[serde(default)]
    pub p2p: P2pConfig,
    #[serde(default)]
    pub fossil: FossilConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeConfig {
    #[serde(default = "default_alias")]
    pub alias: String,
    #[serde(default = "default_log_level")]
    pub log: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct P2pConfig {
    #[serde(default = "default_listen")]
    pub listen: Vec<String>,
    #[serde(default)]
    pub bootstrap_peers: Vec<String>,
    #[serde(default = "default_kad_protocol")]
    pub kad_protocol: String,
    #[serde(default = "default_relay_enabled")]
    pub relay_enabled: bool,
    #[serde(default = "default_idle_timeout_secs")]
    pub idle_timeout_secs: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FossilConfig {
    #[serde(default = "default_fossil_path")]
    pub fossil_path: String,
    #[serde(default = "default_http_port")]
    pub http_port: u16,
    #[serde(default = "default_web_port")]
    pub web_port: u16,
}

fn default_alias() -> String {
    "fossil-p2p-node".to_string()
}
fn default_log_level() -> String {
    "info".to_string()
}
fn default_listen() -> Vec<String> {
    vec!["/ip4/0.0.0.0/tcp/0".to_string()]
}
fn default_kad_protocol() -> String {
    "/fossil-p2p/kad/1.0".to_string()
}
fn default_relay_enabled() -> bool {
    true
}
fn default_idle_timeout_secs() -> u64 {
    120
}
fn default_fossil_path() -> String {
    "fossil".to_string()
}
fn default_http_port() -> u16 {
    8080
}
fn default_web_port() -> u16 {
    3000
}

impl Default for NodeConfig {
    fn default() -> Self {
        Self {
            alias: default_alias(),
            log: default_log_level(),
        }
    }
}

impl Default for P2pConfig {
    fn default() -> Self {
        Self {
            listen: default_listen(),
            bootstrap_peers: vec![],
            kad_protocol: default_kad_protocol(),
            relay_enabled: default_relay_enabled(),
            idle_timeout_secs: default_idle_timeout_secs(),
        }
    }
}

impl Default for FossilConfig {
    fn default() -> Self {
        Self {
            fossil_path: default_fossil_path(),
            http_port: default_http_port(),
            web_port: default_web_port(),
        }
    }
}

impl FossilP2pConfig {
    pub fn load(path: &Path) -> Result<Self> {
        if path.exists() {
            let content = fs::read_to_string(path)?;
            Ok(serde_json::from_str(&content)?)
        } else {
            Ok(Self::default())
        }
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        let content = serde_json::to_string_pretty(self)?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, content)?;
        Ok(())
    }
}
