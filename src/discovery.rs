use crate::config::P2pConfig;
use crate::crypto::Keypair;
use crate::error::{FossilP2pError, Result};
use crate::protocol::{
    decode_advertisement, encode_advertisement, repo_keyspace_key, RepoAdvertisement,
};
use crate::storage::Database;
use libp2p::kad::RecordKey;
use libp2p::{kad, PeerId, Swarm};
use std::time::{Duration, Instant};

/// Advertise local repositories to the Kademlia DHT and persist the signed
/// advertisement in SQLite for later retrieval.
pub fn advertise_local_repos(
    swarm: &mut Swarm<crate::p2p::FossilP2pBehaviour>,
    keypair: &Keypair,
    db: &Database,
    listen_addresses: Vec<String>,
) -> Result<usize> {
    let repos = db.list_repositories()?;
    let mut count = 0usize;

    for repo in &repos {
        let rid = &repo.rid;
        let key = repo_keyspace_key(rid);

        let record = db.load_repository(rid)?;
        let creation_nonce = record
            .as_ref()
            .and_then(|r| r.creation_nonce.clone())
            .unwrap_or_default();

        let owner_did = record
            .as_ref()
            .and_then(|r| crate::crypto::PublicKey::from_hex(&r.owner_key).ok())
            .map(|pk| pk.to_did_key())
            .unwrap_or_else(|| keypair.public_key().to_did_key());

        let ad = RepoAdvertisement::new(
            rid,
            keypair,
            &owner_did,
            &creation_nonce,
            &repo.name,
            repo.description.as_deref().unwrap_or(""),
            &repo.visibility.as_db_str(),
            listen_addresses.clone(),
        );

        let ad_json = encode_advertisement(&ad)?;
        db.advertise_repo(
            rid,
            &keypair.public_key().to_libp2p_peer_id().to_string(),
            Some(&String::from_utf8_lossy(&ad_json)),
        )?;

        // Publish as a DHT record so peers can retrieve the signed advertisement.
        let record = kad::Record {
            key: RecordKey::new(&key),
            value: ad_json,
            publisher: Some(keypair.public_key().to_libp2p_peer_id()),
            expires: Some(Instant::now() + Duration::from_secs(24 * 60 * 60)),
        };
        let _ = swarm
            .behaviour_mut()
            .kad
            .put_record(record, kad::Quorum::One);

        // Also announce via provider records.
        let _ = swarm
            .behaviour_mut()
            .kad
            .start_providing(RecordKey::new(&key));

        count += 1;
    }

    Ok(count)
}

/// Query the DHT for peers providing the given repository RID.
pub fn find_repo_providers(
    swarm: &mut Swarm<crate::p2p::FossilP2pBehaviour>,
    rid: &str,
) -> Result<kad::QueryId> {
    let key = repo_keyspace_key(rid);
    tracing::info!("looking up providers for {rid} on DHT");
    Ok(swarm.behaviour_mut().kad.get_providers(RecordKey::new(&key)))
}

pub fn record_key(rid: &str) -> Result<kad::RecordKey> {
    let key = repo_keyspace_key(rid);
    Ok(kad::RecordKey::new(&key))
}

/// Parse a stored RepoAdvertisement from JSON payload bytes.
pub fn parse_advertisement(bytes: &[u8]) -> Result<RepoAdvertisement> {
    if bytes.len() > 1024 * 1024 {
        return Err(FossilP2pError::Protocol("advertisement too large".into()));
    }
    decode_advertisement(bytes)
}

/// Validate a RepoAdvertisement for a specific RID.
pub fn validate_advertisement_for_rid(ad: &RepoAdvertisement, rid: &str) -> Result<()> {
    if ad.rid != rid {
        return Err(FossilP2pError::Protocol(format!(
            "advertisement RID mismatch: got {} want {}",
            ad.rid, rid
        )));
    }
    if !ad.verify() {
        return Err(FossilP2pError::Protocol(
            "advertisement signature invalid".into(),
        ));
    }
    if !ad.verify_rid() {
        return Err(FossilP2pError::Protocol(
            "advertisement RID does not match owner key + nonce".into(),
        ));
    }
    if !ad.is_fresh(24 * 60 * 60) {
        return Err(FossilP2pError::Protocol("advertisement is stale".into()));
    }
    Ok(())
}

pub fn peer_id_from_public_key(pk: &crate::crypto::PublicKey) -> PeerId {
    pk.to_libp2p_peer_id()
}

/// A discovered provider of a repository.
#[derive(Debug, Clone)]
pub struct Provider {
    pub peer_id: PeerId,
    pub addresses: Vec<String>,
    pub advertisement: Option<RepoAdvertisement>,
}

/// Perform a bounded-time DHT lookup for repository providers.
/// Bootstraps from configured peers, issues a provider query, and collects
/// results until a timeout elapses.
pub async fn lookup_repo_providers(
    p2p_config: &P2pConfig,
    keypair: &libp2p::identity::Keypair,
    rid: &str,
    timeout: Duration,
) -> Result<Vec<Provider>> {
    let mut swarm = crate::p2p::transport::build_oneshot_swarm(p2p_config, keypair)?;

    // Dial bootstrap peers so the routing table is populated.
    for peer_str in &p2p_config.bootstrap_peers {
        if let Ok(multiaddr) = peer_str.parse::<libp2p::Multiaddr>() {
            let peer_id_opt = multiaddr
                .iter()
                .find_map(|p| match p {
                    libp2p::multiaddr::Protocol::P2p(id) => Some(id),
                    _ => None,
                });
            if let Some(peer_id) = peer_id_opt {
                swarm
                    .behaviour_mut()
                    .kad
                    .add_address(&peer_id, multiaddr.clone());
                let _ = swarm.dial(multiaddr);
            }
        }
    }

    let query_id = find_repo_providers(&mut swarm, rid)?;
    let deadline = Instant::now() + timeout;
    let mut providers: Vec<Provider> = Vec::new();
    let mut seen: std::collections::HashSet<PeerId> = std::collections::HashSet::new();
    let mut addresses_for: std::collections::HashMap<PeerId, Vec<String>> =
        std::collections::HashMap::new();

    use futures::StreamExt;
    use libp2p::swarm::SwarmEvent;

    loop {
        if Instant::now() >= deadline {
            break;
        }
        let remaining = deadline.saturating_duration_since(Instant::now());
        match tokio::time::timeout(remaining, swarm.select_next_some()).await {
            Ok(SwarmEvent::Behaviour(
                crate::p2p::behaviour::FossilP2pBehaviourEvent::Kad(
                    kad::Event::OutboundQueryProgressed {
                        id,
                        result: kad::QueryResult::GetProviders(Ok(kad::GetProvidersOk::FoundProviders { providers: peers, .. })),
                        ..
                    },
                ),
            )) if id == query_id => {
                for peer in peers {
                    if seen.insert(peer) {
                        let addresses = addresses_for.get(&peer).cloned().unwrap_or_default();
                        providers.push(Provider {
                            peer_id: peer,
                            addresses,
                            advertisement: None,
                        });
                    }
                }
            }
            Ok(SwarmEvent::Behaviour(
                crate::p2p::behaviour::FossilP2pBehaviourEvent::Identify(
                    libp2p::identify::Event::Received { peer_id, info, .. },
                ),
            )) => {
                let addrs: Vec<String> = info.listen_addrs.iter().map(|a| a.to_string()).collect();
                addresses_for.insert(peer_id, addrs.clone());
                for p in providers.iter_mut() {
                    if p.peer_id == peer_id {
                        p.addresses = addrs.clone();
                    }
                }
            }
            Ok(SwarmEvent::ConnectionEstablished { peer_id, .. }) => {
                tracing::debug!("connected to provider candidate {peer_id}");
            }
            Ok(_) => {}
            Err(_) => break,
        }
    }

    Ok(providers)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::Keypair;

    #[test]
    fn validate_advertisement() {
        let kp = Keypair::generate();
        let nonce = crate::crypto::random_bytes(32);
        let nonce_hex = hex::encode(&nonce);
        let mut nonce_arr = [0u8; 32];
        nonce_arr.copy_from_slice(&nonce);
        let rid = crate::repository::FossilRepoManager::compute_rid(&kp.public_key(), &nonce_arr);

        let ad = RepoAdvertisement::new(
            &rid,
            &kp,
            &kp.public_key().to_did_key(),
            &nonce_hex,
            "repo",
            "",
            "public",
            vec![],
        );
        assert!(validate_advertisement_for_rid(&ad, &rid).is_ok());
        assert!(validate_advertisement_for_rid(&ad, "other-rid").is_err());
    }

    #[test]
    fn reject_tampered_advertisement() {
        let kp = Keypair::generate();
        let mut ad = RepoAdvertisement::new(
            "rid",
            &kp,
            "did",
            "00",
            "repo",
            "",
            "public",
            vec![],
        );
        ad.name = "tampered".into();
        assert!(validate_advertisement_for_rid(&ad, "rid").is_err());
    }
}