use crate::error::{FossilP2pError, Result};
use crate::fossil::FossilCli;
use crate::storage::Database;
use crate::transport::ensure_peer_suffix;

use super::util::{get_config, get_db, get_home, get_keypair};

pub fn cmd_sync(rid: Option<String>) -> Result<()> {
    let home = get_home()?;
    let config = get_config(&home)?;
    let db = get_db(&home)?;

    let rid = match rid {
        Some(r) => r,
        None => db
            .list_repositories()?
            .first()
            .map(|r| r.rid.clone())
            .ok_or_else(|| FossilP2pError::Repository("no repositories found".into()))?,
    };

    let record = db
        .load_repository(&rid)?
        .ok_or_else(|| FossilP2pError::Repository(format!("repository not found: {rid}")))?;

    println!("Syncing: {} ({})", record.name, rid);

    let keypair = get_keypair(&home)?;
    let libp2p_keypair = keypair
        .to_libp2p_keypair()
        .map_err(|e| FossilP2pError::P2p(format!("key conversion: {e}")))?;

    // Find a remote provider (prefer one that is not us).
    let our_peer_id = keypair.public_key().to_libp2p_peer_id().to_string();
    let (provider_id, provider_addr) =
        find_sync_provider(&config, &libp2p_keypair, &rid, &our_peer_id, &db)?;

    db.store_peer(
        &provider_id.to_string(),
        &provider_id.to_string(),
        None,
        Some(&provider_addr.to_string()),
    )?;
    db.advertise_repo(&rid, &provider_id.to_string(), None)?;

    let transport_url = format!("http://{}/{}", crate::transport::RESOLVE_HOSTS[0], rid);
    println!("  Remote: {provider_id} @ {provider_addr}");

    let fossil = FossilCli::new(&config.fossil);
    let output = fossil.sync(std::path::Path::new(&record.path), Some(&transport_url))?;

    if !output.trim().is_empty() {
        println!("{output}");
    }
    println!("Sync complete.");
    Ok(())
}

fn find_sync_provider(
    config: &crate::config::FossilP2pConfig,
    libp2p_keypair: &libp2p::identity::Keypair,
    rid: &str,
    our_peer_id: &str,
    db: &Database,
) -> Result<(libp2p::PeerId, libp2p::Multiaddr)> {
    // Locally known providers first.
    for advertised in db.find_repo_providers(rid)? {
        if advertised.peer_id == our_peer_id {
            continue;
        }
        let peer_id = match advertised.peer_id.parse::<libp2p::PeerId>() {
            Ok(id) => id,
            Err(_) => continue,
        };
        if let Ok(Some(peer)) = db.get_peer(&advertised.peer_id) {
            if let Some(addrs) = peer.addresses {
                for addr_str in addrs.split(',') {
                    let addr_str = addr_str.trim();
                    if addr_str.is_empty() {
                        continue;
                    }
                    if let Ok(address) = addr_str.parse::<libp2p::Multiaddr>() {
                        return Ok((peer_id, ensure_peer_suffix(address, peer_id)));
                    }
                }
            }
        }
    }

    // Fall back to DHT lookup (blocking on a bounded async query).
    let providers = tokio::runtime::Runtime::new()?.block_on(async {
        crate::discovery::lookup_repo_providers(
            &config.p2p,
            libp2p_keypair,
            rid,
            std::time::Duration::from_secs(25),
        )
        .await
    })?;

    providers
        .into_iter()
        .filter(|p| p.peer_id.to_string() != our_peer_id)
        .find_map(|p| {
            p.addresses
                .first()
                .and_then(|a| a.parse::<libp2p::Multiaddr>().ok())
                .map(|addr| (p.peer_id, ensure_peer_suffix(addr, p.peer_id)))
        })
        .ok_or_else(|| {
            FossilP2pError::P2p(format!(
                "no remote provider for {rid}; start a node that publishes it or pass --peer"
            ))
        })
}
