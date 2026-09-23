use crate::config::FossilP2pConfig;
use crate::crypto::PublicKey;
use crate::error::{FossilP2pError, Result};
use crate::fossil::FossilCli;
use crate::identity::Visibility;
use crate::repository::FossilRepoManager;
use std::path::{Path, PathBuf};

use super::util::{default_repo_name, get_config, get_db, get_home, get_keypair};
use crate::transport::ensure_peer_suffix;

pub fn cmd_repo_list() -> Result<()> {
    let home = get_home()?;
    let db = get_db(&home)?;
    let repos = db.list_repositories()?;
    if repos.is_empty() {
        println!("No published repositories.");
        return Ok(());
    }
    println!("{:<66} {:<30} {:<12}", "RID", "NAME", "VISIBILITY");
    println!("{}", "-".repeat(110));
    for repo in &repos {
        let rid_short = if repo.rid.len() > 64 {
            format!("{}...", &repo.rid[..61])
        } else {
            repo.rid.clone()
        };
        let name_short = if repo.name.len() > 29 {
            format!("{}...", &repo.name[..26])
        } else {
            repo.name.clone()
        };
        println!(
            "{:<66} {:<30} {:<12}",
            rid_short,
            name_short,
            repo.visibility.as_db_str()
        );
    }
    Ok(())
}

pub fn cmd_repo_init(
    path: PathBuf,
    name: Option<String>,
    description: Option<String>,
    visibility: Option<String>,
) -> Result<()> {
    let vis = visibility
        .as_deref()
        .map(crate::cli::visibility_from_str)
        .unwrap_or(Visibility::Public);
    register_repo(path, name, description, vis, true)
}

pub fn cmd_repo_publish(
    path: PathBuf,
    name: Option<String>,
    description: Option<String>,
    visibility: Option<String>,
) -> Result<()> {
    let vis = visibility
        .as_deref()
        .map(crate::cli::visibility_from_str)
        .unwrap_or(Visibility::Public);
    register_repo(path, name, description, vis, false)
}

fn register_repo(
    repo_path: PathBuf,
    name: Option<String>,
    description: Option<String>,
    visibility: Visibility,
    create_new: bool,
) -> Result<()> {
    let home = get_home()?;
    home.init()?;
    let keypair = get_keypair(&home)?;
    let pk = keypair.public_key();
    let config = get_config(&home)?;
    let db = get_db(&home)?;

    let repo_name = name.unwrap_or_else(|| default_repo_name(&repo_path));
    let desc = description.unwrap_or_default();

    let fossil = FossilCli::new(&config.fossil);
    let manager = FossilRepoManager::new(fossil);

    let repo_identity = if create_new {
        manager.init_repo(&repo_path, &repo_name, &desc, &keypair, visibility)?
    } else {
        if !repo_path.exists() {
            return Err(FossilP2pError::Repository(format!(
                "path does not exist: {}",
                repo_path.display()
            )));
        }
        manager.publish_repo(&repo_path, &repo_name, &desc, &keypair, visibility)?
    };

    let fossil_db = crate::fossil::find_fossil_file(&repo_path);
    db.store_repository(
        &repo_identity.rid,
        &repo_identity.name,
        Some(&repo_identity.description),
        &repo_path.to_string_lossy(),
        &pk.to_hex(),
        &repo_identity.visibility,
        fossil_db.as_ref().map(|p| p.to_string_lossy()).as_deref(),
        repo_identity.protocol_version,
    )?;
    db.store_repository_identity(
        &repo_identity.rid,
        &repo_identity.creation_nonce,
        repo_identity.protocol_version,
    )?;

    let verb = if create_new { "initialized" } else { "published" };
    println!("Repository {verb}!");
    println!("  RID:        {}", repo_identity.rid);
    println!("  Name:       {}", repo_identity.name);
    println!("  Owner:      {}", repo_identity.owner_did);
    println!("  Visibility: {}", repo_identity.visibility.as_db_str());
    println!("  Path:       {}", repo_path.display());
    println!();
    println!("  Announce it to the network by running: peergit node start");
    Ok(())
}

pub fn cmd_repo_unpublish(rid: String) -> Result<()> {
    let home = get_home()?;
    let db = get_db(&home)?;
    let record = db.load_repository(&rid)?.ok_or_else(|| {
        FossilP2pError::Repository(format!("repository not found: {rid}"))
    })?;
    db.delete_repository(&rid)?;
    println!("Repository unpublished: {} ({})", record.name, rid);
    Ok(())
}

pub fn cmd_repo_visibility(rid: String, visibility: String) -> Result<()> {
    let home = get_home()?;
    let db = get_db(&home)?;
    let record = db.load_repository(&rid)?.ok_or_else(|| {
        FossilP2pError::Repository(format!("repository not found: {rid}"))
    })?;

    // Update the on-disk identity file so it stays authoritative.
    let repo_path = Path::new(&record.path);
    if repo_path.join(".fossil-p2p-identity.json").exists() {
        let vis = crate::cli::visibility_from_str(&visibility);
        let config = get_config(&home)?;
        let manager = FossilRepoManager::new(FossilCli::new(&config.fossil));
        manager.update_metadata(repo_path, None, None, Some(&vis))?;
    }

    let vis = crate::cli::visibility_from_str(&visibility);
    db.update_repository_visibility(&rid, &vis)?;
    println!("Visibility updated: {} -> {visibility}", record.name);
    Ok(())
}

pub fn cmd_repo_discover(rid: String) -> Result<()> {
    let home = get_home()?;
    let db = get_db(&home)?;

    if let Some(record) = db.load_repository(&rid)? {
        println!("Repository found locally:");
        println!("  RID:        {}", record.rid);
        println!("  Name:       {}", record.name);
        if let Some(d) = &record.description {
            if !d.is_empty() {
                println!("  Description: {d}");
            }
        }
        println!("  Path:       {}", record.path);
        println!("  Visibility: {}", record.visibility.as_db_str());
    } else {
        println!("Repository not found locally. Querying the DHT...");
    }

    let keypair = get_keypair(&home)?;
    let config = get_config(&home)?;
    let libp2p_keypair = keypair
        .to_libp2p_keypair()
        .map_err(|e| FossilP2pError::P2p(format!("key conversion: {e}")))?;

    let providers = tokio::runtime::Runtime::new()?
        .block_on(async {
            crate::discovery::lookup_repo_providers(
                &config.p2p,
                &libp2p_keypair,
                &rid,
                std::time::Duration::from_secs(20),
            )
            .await
        })?;

    if providers.is_empty() {
        println!("No providers found on the DHT for {rid}.");
        return Ok(());
    }

    println!("\nProviders found on the DHT:");
    for p in &providers {
        println!("  Peer ID:   {}", p.peer_id);
        if p.addresses.is_empty() {
            println!("    Addresses: (unknown - will be discovered on connect)");
        } else {
            for a in &p.addresses {
                println!("    Address:   {a}");
            }
        }
    }
    Ok(())
}

pub fn cmd_repo_clone(rid: String, directory: PathBuf, peer: Option<String>) -> Result<()> {
    let home = get_home()?;
    home.init()?;
    let keypair = get_keypair(&home)?;
    let config = get_config(&home)?;
    let db = get_db(&home)?;

    if db.load_repository(&rid)?.is_some() {
        println!("Note: repository {rid} is already known locally.");
    }

    let libp2p_keypair = keypair
        .to_libp2p_keypair()
        .map_err(|e| FossilP2pError::P2p(format!("key conversion: {e}")))?;

    let provider = if let Some(peer) = &peer {
        let (peer_id, addr) = crate::transport::resolve_peer(&home, peer)?;
        DiscoveredProvider {
            peer_id,
            address: addr,
        }
    } else {
        tokio::runtime::Runtime::new()?.block_on(async {
            discover_best_provider(&config, &libp2p_keypair, &rid, &home).await
        })?
    };

    println!("Cloning {rid} from {} ...", provider.peer_id);

    let target = if directory.as_os_str() == "." {
        PathBuf::from(repo_name_hint(&rid))
    } else {
        directory
    };

    // Register the provider so the in-process transport command (spawned by
    // Fossil) can resolve it from the RID carried in the transport URL.
    db.store_peer(
        &provider.peer_id.to_string(),
        &provider.peer_id.to_string(),
        None,
        Some(&provider.address.to_string()),
    )?;
    db.advertise_repo(&rid, &provider.peer_id.to_string(), None)?;

    let transport_url = format!("http://{}/{}", crate::transport::RESOLVE_HOSTS[0], rid);
    let fossil = FossilCli::new(&config.fossil);
    fossil.clone(&transport_url, &target)?;

    // Reconstruct the PeerGit identity file from the remote signed advertisement.
    let advertisement = tokio::runtime::Runtime::new()?.block_on(async {
        crate::transport::fetch_advertisement(
            &config,
            &keypair,
            &libp2p_keypair,
            provider.peer_id,
            &rid,
        )
        .await
    })?;

    if let Some(ad) = advertisement {
        crate::discovery::validate_advertisement_for_rid(&ad, &rid)?;
        let identity = ad.to_identity();
        let identity_path = target.join(".fossil-p2p-identity.json");
        std::fs::write(&identity_path, identity.to_json()?)?;

        db.store_repository(
            &ad.rid,
            &ad.name,
            Some(&ad.description),
            &target.to_string_lossy(),
            &ad.owner.to_hex(),
            &crate::identity::Visibility::from_db_str(&ad.visibility),
            None,
            ad.protocol_version,
        )?;
        db.store_repository_identity(&ad.rid, &ad.creation_nonce, ad.protocol_version)?;

        println!("Repository cloned and identity verified!");
        println!("  RID:    {}", ad.rid);
        println!("  Owner:  {}", ad.owner_did);
        println!("  Path:   {}", target.display());
    } else {
        println!("Repository cloned (no signed advertisement available).");
        println!("  RID:    {rid}");
        println!("  Path:   {}", target.display());
    }

    Ok(())
}

struct DiscoveredProvider {
    peer_id: libp2p::PeerId,
    address: libp2p::Multiaddr,
}

async fn discover_best_provider(
    config: &FossilP2pConfig,
    keypair: &libp2p::identity::Keypair,
    rid: &str,
    home: &crate::home::Home,
) -> Result<DiscoveredProvider> {
    let db = crate::storage::Database::open(&home.db())?;

    // Prefer locally-known providers first.
    for record in db.find_repo_providers(rid)? {
        if let Ok(peer_id) = record.peer_id.parse::<libp2p::PeerId>() {
            if let Ok(Some(known)) = db.get_peer(&record.peer_id) {
                if let Some(addrs) = known.addresses {
                    for addr_str in addrs.split(',') {
                        if let Ok(address) = addr_str.trim().parse::<libp2p::Multiaddr>() {
                            let address = ensure_peer_suffix(address, peer_id);
                            return Ok(DiscoveredProvider { peer_id, address });
                        }
                    }
                }
            }
        }
    }

    let providers = crate::discovery::lookup_repo_providers(
        &config.p2p,
        keypair,
        rid,
        std::time::Duration::from_secs(25),
    )
    .await?;

    for p in providers {
        if let Some(addr_str) = p.addresses.first() {
            if let Ok(address) = addr_str.parse::<libp2p::Multiaddr>() {
                let with_peer = ensure_peer_suffix(address, p.peer_id);
                return Ok(DiscoveredProvider {
                    peer_id: p.peer_id,
                    address: with_peer,
                });
            }
        }
    }

    Err(FossilP2pError::P2p(format!(
        "no reachable provider found for {rid} on the DHT"
    )))
}

fn repo_name_hint(rid: &str) -> String {
    let short = if rid.len() > 12 { &rid[..12] } else { rid };
    format!("repo-{short}")
}

/// Resolve the owner public key of a locally known repository (used in tests).
pub fn local_owner_key(home: &crate::home::Home, rid: &str) -> Result<Option<PublicKey>> {
    let db = get_db(home)?;
    Ok(db
        .load_repository(rid)?
        .and_then(|r| PublicKey::from_hex(&r.owner_key).ok()))
}