use crate::crypto::PublicKey;
use crate::storage::Database;
use crate::web::WebState;
use std::path::Path;

type WebResult<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;

pub fn node_status(state: &WebState) -> WebResult<String> {
    let keypair = crate::keystore::load_or_create_keypair(&state.home)?;
    let pk = keypair.public_key();
    let peer_id = pk.to_libp2p_peer_id().to_string();

    let db = Database::open(&state.home.db())?;
    let peer_count = db.list_peers().map(|p| p.len()).unwrap_or(0);
    let repo_count = db.list_repositories().map(|r| r.len()).unwrap_or(0);
    let advertised = db
        .list_advertised_repos(&peer_id)
        .map(|r| r.len())
        .unwrap_or(0);

    let status = serde_json::json!({
        "alias": state.config.node.alias,
        "peer_id": peer_id,
        "public_key": pk.to_multibase(),
        "did": pk.to_did_key(),
        "peer_count": peer_count,
        "repo_count": repo_count,
        "advertised_count": advertised,
        "listen": state.config.p2p.listen.join(", "),
        "web_port": state.config.fossil.web_port,
        "protocol_version": crate::protocol::PROTOCOL_VERSION,
    });

    Ok(serde_json::to_string(&status)?)
}

pub fn list_peers(state: &WebState) -> WebResult<String> {
    let db = Database::open(&state.home.db())?;
    let peers = db.list_peers()?;

    let items: Vec<serde_json::Value> = peers
        .into_iter()
        .map(|p| {
            serde_json::json!({
                "peer_id": p.peer_id,
                "public_key": p.public_key,
                "alias": p.alias,
                "addresses": p.addresses,
                "first_seen": p.first_seen,
                "last_seen": p.last_seen,
            })
        })
        .collect();

    Ok(serde_json::to_string(&items)?)
}

pub fn list_repos(state: &WebState) -> WebResult<String> {
    let db = Database::open(&state.home.db())?;
    let repos = db.list_repositories()?;

    let items: Vec<serde_json::Value> = repos
        .into_iter()
        .map(|r| {
            serde_json::json!({
                "rid": r.rid,
                "name": r.name,
                "description": r.description,
                "visibility": r.visibility.as_db_str(),
            })
        })
        .collect();

    Ok(serde_json::to_string(&items)?)
}

pub fn add_peer(state: &WebState, body: &[u8]) -> WebResult<String> {
    let v: serde_json::Value = serde_json::from_slice(body)?;
    let pk_str = v["public_key"].as_str().ok_or("public_key required")?;
    let alias = v["alias"].as_str();

    let pk = PublicKey::from_multibase(pk_str)?;
    let peer_id = pk.to_libp2p_peer_id().to_string();

    let db = Database::open(&state.home.db())?;
    db.store_peer(&peer_id, &pk.to_hex(), alias, None)?;

    Ok(serde_json::json!({"ok": true, "peer_id": peer_id}).to_string())
}

pub fn trigger_sync(state: &WebState) -> WebResult<String> {
    let db = Database::open(&state.home.db())?;
    let repos = db.list_repositories()?;

    if repos.is_empty() {
        return Ok(serde_json::json!({"ok": false, "error": "no repositories"}).to_string());
    }

    let fossil = crate::fossil::FossilCli::new(&state.config.fossil);
    let mut results = Vec::new();

    for summary in &repos {
        let record = match db.load_repository(&summary.rid)? {
            Some(r) => r,
            None => continue,
        };

        let repo_path = Path::new(&record.path);
        if !repo_path.exists() {
            results.push(serde_json::json!({
                "rid": summary.rid, "name": summary.name, "ok": false,
                "error": "repository path missing"
            }));
            continue;
        }

        match fossil.sync(repo_path, None) {
            Ok(output) => {
                results.push(serde_json::json!({
                    "rid": summary.rid, "name": summary.name, "ok": true,
                    "output": output.trim()
                }));
            }
            Err(e) => {
                results.push(serde_json::json!({
                    "rid": summary.rid, "name": summary.name, "ok": false,
                    "error": e.to_string()
                }));
            }
        }
    }

    Ok(serde_json::json!({"ok": true, "results": results}).to_string())
}

pub fn advertised_repos(state: &WebState) -> WebResult<String> {
    let keypair = crate::keystore::load_or_create_keypair(&state.home)?;
    let peer_id = keypair.public_key().to_libp2p_peer_id().to_string();
    let db = Database::open(&state.home.db())?;
    let ads = db.list_advertised_repos(&peer_id)?;
    let items: Vec<serde_json::Value> = ads
        .into_iter()
        .map(|a| {
            serde_json::json!({
                "rid": a.rid,
                "peer_id": a.peer_id,
                "announced_at": a.announced_at,
                "has_advertisement": a.advertisement_json.is_some(),
            })
        })
        .collect();
    Ok(serde_json::to_string(&items)?)
}
