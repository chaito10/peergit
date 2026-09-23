use crate::crypto::PublicKey;
use crate::error::{FossilP2pError, Result};

use super::util::{get_db, get_home};

pub fn cmd_peer_add(
    public_key: String,
    alias: Option<String>,
    addresses: Vec<String>,
) -> Result<()> {
    let home = get_home()?;
    let db = get_db(&home)?;
    let pk = PublicKey::from_multibase(&public_key)?;
    let peer_id = pk.to_libp2p_peer_id().to_string();
    let address_str = if addresses.is_empty() {
        None
    } else {
        Some(addresses.join(","))
    };
    db.store_peer(&peer_id, &pk.to_hex(), alias.as_deref(), address_str.as_deref())?;
    println!("Peer added: {}", pk);
    println!("  Peer ID: {}", peer_id);
    if let Some(a) = &alias {
        println!("  Alias:   {}", a);
    }
    if !addresses.is_empty() {
        println!("  Addresses: {}", addresses.join(", "));
    }
    Ok(())
}

pub fn cmd_peer_list() -> Result<()> {
    let home = get_home()?;
    let db = get_db(&home)?;
    let peers = db.list_peers()?;
    if peers.is_empty() {
        println!("No known peers.");
        return Ok(());
    }
    println!(
        "{:<65} {:<25} {:<20} {:<30}",
        "PEER ID", "PUBLIC KEY", "ALIAS", "LAST SEEN"
    );
    println!("{}", "-".repeat(142));
    for peer in &peers {
        let alias_str = peer.alias.as_deref().unwrap_or("-");
        let pk_short = if peer.public_key.len() > 24 {
            format!("{}...", &peer.public_key[..21])
        } else {
            peer.public_key.clone()
        };
        let last_short = if peer.last_seen.len() > 10 {
            &peer.last_seen[..10]
        } else {
            &peer.last_seen
        };
        println!(
            "{:<65} {:<25} {:<20} {:<30}",
            peer.peer_id, pk_short, alias_str, last_short
        );
    }
    Ok(())
}

pub fn cmd_peer_remove(peer_id: String) -> Result<()> {
    let home = get_home()?;
    let db = get_db(&home)?;
    let removed = db
        .conn
        .execute("DELETE FROM known_peers WHERE peer_id = ?1", rusqlite::params![peer_id])?;
    if removed == 0 {
        return Err(FossilP2pError::P2p(format!("peer not found: {peer_id}")));
    }
    println!("Peer removed: {peer_id}");
    Ok(())
}