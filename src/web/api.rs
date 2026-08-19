use crate::crypto::PublicKey;
use crate::home::Home;
use crate::storage::Database;
use crate::web::WebState;
use std::path::Path;

pub fn node_status(state: &WebState) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let keypair = load_keypair(&state.home)?;
    let pk = keypair.public_key();
    let peer_id = pk.to_libp2p_peer_id().to_string();

    let peer_count = Database::open(&state.home.db())
        .ok()
        .and_then(|db| db.list_peers().ok())
        .map(|p| p.len())
        .unwrap_or(0);

    let repo_count = Database::open(&state.home.db())
        .ok()
        .and_then(|db| db.list_repositories().ok())
        .map(|r| r.len())
        .unwrap_or(0);

    let status = serde_json::json!({
        "alias": state.config.node.alias,
        "peer_id": peer_id,
        "public_key": pk.to_multibase(),
        "peer_count": peer_count,
        "repo_count": repo_count,
        "listen": state.config.p2p.listen.join(", "),
        "web_port": state.config.fossil.http_port,
    });

    Ok(serde_json::to_string(&status)?)
}

pub fn list_peers(state: &WebState) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let db = Database::open(&state.home.db())?;
    let peers = db.list_peers()?;

    let items: Vec<serde_json::Value> = peers
        .into_iter()
        .map(|(peer_id, _pk, alias, addresses, last_seen)| {
            serde_json::json!({
                "peer_id": peer_id,
                "alias": alias,
                "addresses": addresses,
                "last_seen": last_seen,
            })
        })
        .collect();

    Ok(serde_json::to_string(&items)?)
}

pub fn list_repos(state: &WebState) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let db = Database::open(&state.home.db())?;
    let repos = db.list_repositories()?;

    let items: Vec<serde_json::Value> = repos
        .into_iter()
        .map(|(rid, name, description, visibility)| {
            serde_json::json!({
                "rid": rid,
                "name": name,
                "description": description,
                "visibility": visibility,
            })
        })
        .collect();

    Ok(serde_json::to_string(&items)?)
}

pub fn add_peer(
    state: &WebState,
    body: &[u8],
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let v: serde_json::Value = serde_json::from_slice(body)?;
    let pk_str = v["public_key"]
        .as_str()
        .ok_or("public_key required")?;
    let alias = v["alias"].as_str();

    let pk = PublicKey::from_multibase(pk_str)?;
    let peer_id = pk.to_libp2p_peer_id().to_string();

    let db = Database::open(&state.home.db())?;
    db.store_peer(&peer_id, &pk.to_hex(), alias, None)?;

    Ok(serde_json::json!({"ok": true, "peer_id": peer_id}).to_string())
}

pub fn trigger_sync(
    state: &WebState,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let db = Database::open(&state.home.db())?;
    let repos = db.list_repositories()?;

    if repos.is_empty() {
        return Ok(serde_json::json!({"ok": false, "error": "no repositories"}).to_string());
    }

    let fossil = crate::fossil::FossilCli::new(&state.config.fossil);
    let mut results = Vec::new();

    for (rid, name, _desc, _vis) in &repos {
        let (_name, _desc, path, _owner, _vis, _fossil_db) =
            db.load_repository(rid)?.unwrap_or_default();

        let repo_path = Path::new(&path);
        if repo_path.exists() {
            match fossil.sync(repo_path, None) {
                Ok(output) => {
                    results.push(serde_json::json!({
                        "rid": rid, "name": name, "ok": true,
                        "output": output.trim()
                    }));
                }
                Err(e) => {
                    results.push(serde_json::json!({
                        "rid": rid, "name": name, "ok": false,
                        "error": e.to_string()
                    }));
                }
            }
        }
    }

    Ok(serde_json::json!({"ok": true, "results": results}).to_string())
}

fn load_keypair(home: &Home) -> Result<crate::crypto::Keypair, Box<dyn std::error::Error + Send + Sync>> {
    let sk_path = home.secret_key_path();
    let sk_hex = std::fs::read_to_string(&sk_path)
        .map_err(|e| format!("cannot read key at {}: {e}", sk_path.display()))?;
    let sk_bytes = hex::decode(sk_hex.trim())?;
    let sk_arr: [u8; 32] = sk_bytes.try_into().map_err(|_| "invalid key length")?;
    Ok(crate::crypto::Keypair::from_bytes(&sk_arr)?)
}
