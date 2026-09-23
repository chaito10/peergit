use crate::config::FossilP2pConfig;
use crate::error::{FossilP2pError, Result};

use super::util::{get_config, get_home};

pub fn cmd_config_show() -> Result<()> {
    let home = get_home()?;
    let config = get_config(&home)?;
    println!("{}", serde_json::to_string_pretty(&config)?);
    Ok(())
}

pub fn cmd_config_init() -> Result<()> {
    let home = get_home()?;
    home.init()?;
    let config = FossilP2pConfig::default();
    config.save(&home.config())?;
    println!("Configuration initialized at {}", home.config().display());
    Ok(())
}

pub fn cmd_config_get(key: String) -> Result<()> {
    let home = get_home()?;
    let config = get_config(&home)?;
    let value = match key.as_str() {
        "node.alias" => config.node.alias.clone(),
        "node.log" => config.node.log.clone(),
        "p2p.listen" => config.p2p.listen.join(", "),
        "p2p.bootstrap_peers" => config.p2p.bootstrap_peers.join(", "),
        "p2p.kad_protocol" => config.p2p.kad_protocol.clone(),
        "p2p.idle_timeout_secs" => config.p2p.idle_timeout_secs.to_string(),
        "fossil.fossil_path" => config.fossil.fossil_path.clone(),
        "fossil.http_port" => config.fossil.http_port.to_string(),
        "fossil.web_port" => config.fossil.web_port.to_string(),
        _ => {
            return Err(FossilP2pError::Config(format!("unknown key: {key}")));
        }
    };
    println!("{value}");
    Ok(())
}

pub fn cmd_config_set(key: String, value: String) -> Result<()> {
    let home = get_home()?;
    let mut config = get_config(&home)?;
    match key.as_str() {
        "node.alias" => config.node.alias = value,
        "node.log" => config.node.log = value,
        "p2p.listen" => {
            config.p2p.listen = value.split(',').map(|s| s.trim().to_string()).collect()
        }
        "p2p.bootstrap_peers" => {
            config.p2p.bootstrap_peers =
                value.split(',').map(|s| s.trim().to_string()).collect()
        }
        "p2p.kad_protocol" => config.p2p.kad_protocol = value,
        "p2p.idle_timeout_secs" => {
            config.p2p.idle_timeout_secs = value
                .parse()
                .map_err(|e| FossilP2pError::Config(format!("invalid u64: {e}")))?
        }
        "fossil.fossil_path" => config.fossil.fossil_path = value,
        "fossil.http_port" => {
            config.fossil.http_port = value
                .parse()
                .map_err(|e| FossilP2pError::Config(format!("invalid u16: {e}")))?
        }
        "fossil.web_port" => {
            config.fossil.web_port = value
                .parse()
                .map_err(|e| FossilP2pError::Config(format!("invalid u16: {e}")))?
        }
        _ => {
            return Err(FossilP2pError::Config(format!("unknown key: {key}")));
        }
    }
    config.save(&home.config())?;
    println!("Configuration updated.");
    Ok(())
}