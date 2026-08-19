use crate::config::P2pConfig;
use crate::error::Result;
use crate::p2p::behaviour::FossilP2pBehaviour;
use libp2p::{noise, tcp, yamux, Swarm, SwarmBuilder};
use std::time::Duration;

pub fn build_swarm(
    config: &P2pConfig,
    keypair: &libp2p::identity::Keypair,
) -> Result<Swarm<FossilP2pBehaviour>> {
    let kad_protocol = config.kad_protocol.clone();
    let idle_timeout = Duration::from_secs(config.idle_timeout_secs);

    let mut swarm = SwarmBuilder::with_existing_identity(keypair.clone())
        .with_tokio()
        .with_tcp(
            tcp::Config::default(),
            noise::Config::new,
            yamux::Config::default,
        )
        .map_err(|e| crate::error::FossilP2pError::P2p(format!("tcp setup: {e}")))?
        .with_behaviour(|key| {
            let peer_id = key.public().to_peer_id();
            Ok(FossilP2pBehaviour::new(peer_id, key, &kad_protocol))
        })
        .map_err(|e| crate::error::FossilP2pError::P2p(format!("behaviour setup: {e}")))?
        .with_swarm_config(|cfg| cfg.with_idle_connection_timeout(idle_timeout))
        .build();

    for addr_str in &config.listen {
        if let Ok(addr) = addr_str.parse() {
            if let Err(e) = swarm.listen_on(addr) {
                eprintln!("Warning: failed to listen on {addr_str}: {e}");
            }
        }
    }

    Ok(swarm)
}
