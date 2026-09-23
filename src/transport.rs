use crate::config::FossilP2pConfig;
use crate::crypto::{Keypair, PublicKey};
use crate::error::{FossilP2pError, Result};
use crate::home::Home;
use crate::p2p::behaviour::{FossilP2pBehaviour, FossilP2pBehaviourEvent};
use crate::p2p::transfer::{
    build_xfer_error, build_xfer_request, build_xfer_response, check_peer_authorized,
    parse_xfer_envelope,
};
use crate::protocol::Operation;
use crate::storage::Database;
use futures::StreamExt;
use libp2p::{Multiaddr, PeerId, Swarm, request_response, swarm::SwarmEvent};
use std::fs;
use std::path::Path;
use std::time::{Duration, Instant};

pub const TRANSPORT_SCHEME: &str = "peergit";

/// Placeholder hosts that mean "resolve the provider by RID" rather than
/// naming a specific peer. Fossil only treats `http(s)://` URLs as remote when
/// using `--transport-command`, so peerless URLs use one of these hosts.
pub const RESOLVE_HOSTS: &[&str] = &["peergit", "peergit.local", "peergit.test"];

/// A parsed PeerGit transfer target: which peer to contact and which RID to request.
#[derive(Debug, Clone)]
pub struct TransportTarget {
    pub peer: String,
    pub rid: String,
}

/// Parse a transport URL of the form `peergit://<peer>/<rid>`.
///
/// `<peer>` may be a libp2p multiaddr (e.g. `/ip4/1.2.3.4/tcp/4001/p2p/<id>`),
/// a bare PeerId, a locally-known peer alias, or one of [`RESOLVE_HOSTS`] to
/// resolve the provider from the RID. `http://` and `https://` URLs are also
/// accepted because Fossil requires an HTTP-like scheme for
/// `--transport-command`. Fossil appends its method segment (`/xfer`) before
/// invoking the transport command; this is stripped here.
pub fn parse_transport_url(url: &str) -> Result<TransportTarget> {
    let rest = url
        .strip_prefix("peergit://")
        .or_else(|| url.strip_prefix("http://"))
        .or_else(|| url.strip_prefix("https://"))
        .ok_or_else(|| {
            FossilP2pError::Protocol(format!(
                "unsupported transport scheme: {url} (expected peergit://<peer>/<rid> or http://<host>/<rid>)"
            ))
        })?;

    let rest = rest.split(['?', '#']).next().unwrap_or(rest);
    let mut rest = rest.trim_end_matches('/');
    if let Some(stripped) = rest.strip_suffix("/xfer") {
        rest = stripped;
    }

    // A multiaddr always begins with '/', so the trailing path segment is the
    // RID (a 64-char hex digest) when present.
    if rest.starts_with('/') {
        if let Some(idx) = rest.rfind('/') {
            let tail = &rest[idx + 1..];
            if is_rid(tail) {
                return Ok(TransportTarget {
                    peer: rest[..idx].to_string(),
                    rid: tail.to_string(),
                });
            }
        }
        return Ok(TransportTarget {
            peer: rest.to_string(),
            rid: String::new(),
        });
    }

    let mut parts = rest.splitn(2, '/');
    let first = parts.next().unwrap_or("");
    let second = parts.next();

    match second {
        Some(rid) if !rid.is_empty() => Ok(TransportTarget {
            peer: first.to_string(),
            rid: rid.to_string(),
        }),
        Some(_) => Ok(TransportTarget {
            peer: first.to_string(),
            rid: String::new(),
        }),
        None => Ok(TransportTarget {
            peer: String::new(),
            rid: first.to_string(),
        }),
    }
}

/// Repository IDs are lowercase SHA-256 digests rendered as 64 hex characters.
pub fn is_rid(value: &str) -> bool {
    value.len() == 64 && value.chars().all(|c| c.is_ascii_hexdigit())
}

/// Sender side: invoked by Fossil's `--transport-command`.
pub fn run_transport(url: &str, request_file: &Path, reply_file: &Path) -> Result<()> {
    let request = fs::read(request_file).map_err(|e| {
        FossilP2pError::P2p(format!(
            "failed to read request file {}: {e}",
            request_file.display()
        ))
    })?;

    let target = parse_transport_url(url)?;

    let home = Home::new()?;
    let config = FossilP2pConfig::load(&home.config())?;
    let keypair = load_keypair(&home)?;
    let libp2p_keypair = keypair
        .to_libp2p_keypair()
        .map_err(|e| FossilP2pError::P2p(format!("key conversion: {e}")))?;

    let (peer_id, address) = resolve_target(&home, &config, &libp2p_keypair, &target)?;

    let operation = if request.starts_with(b"GET ") {
        Operation::Fetch
    } else {
        Operation::Push
    };

    let (envelope, expected_req_id) =
        build_xfer_request(&keypair, &target.rid, operation, request)?;

    let response = tokio::runtime::Runtime::new()
        .map_err(|e| FossilP2pError::P2p(format!("tokio runtime: {e}")))?
        .block_on(run_oneshot_transfer(
            &config,
            &libp2p_keypair,
            peer_id,
            address,
            envelope,
            expected_req_id,
        ))?;

    let (_env, xfer) = crate::p2p::transfer::parse_xfer_response(&response)?;
    if !xfer.ok {
        return Err(FossilP2pError::P2p(format!(
            "remote rejected transfer: {}",
            xfer.error.unwrap_or_else(|| "unknown error".into())
        )));
    }

    fs::write(reply_file, &xfer.payload).map_err(|e| {
        FossilP2pError::P2p(format!(
            "failed to write reply file {}: {e}",
            reply_file.display()
        ))
    })?;

    Ok(())
}

async fn run_oneshot_transfer(
    config: &FossilP2pConfig,
    keypair: &libp2p::identity::Keypair,
    peer_id: PeerId,
    address: Multiaddr,
    envelope: Vec<u8>,
    expected_req_id: [u8; 16],
) -> Result<Vec<u8>> {
    let mut swarm = build_oneshot_swarm(config, keypair)?;

    swarm
        .dial(address)
        .map_err(|e| FossilP2pError::P2p(format!("dial failed: {e}")))?;

    let mut connected = false;
    let mut request_id: Option<request_response::OutboundRequestId> = None;
    let deadline = Instant::now() + Duration::from_secs(120);

    loop {
        if Instant::now() > deadline {
            return Err(FossilP2pError::P2p("timeout waiting for response".into()));
        }

        match swarm.select_next_some().await {
            SwarmEvent::ConnectionEstablished { peer_id: p, .. } if p == peer_id => {
                tracing::info!("connected to {peer_id}");
                let id = swarm
                    .behaviour_mut()
                    .xfer
                    .send_request(&peer_id, envelope.clone());
                request_id = Some(id);
                connected = true;
            }
            SwarmEvent::OutgoingConnectionError { error, .. } => {
                return Err(FossilP2pError::P2p(format!("connection error: {error}")));
            }
            SwarmEvent::Behaviour(FossilP2pBehaviourEvent::Xfer(
                request_response::Event::Message {
                    message:
                        request_response::Message::Response {
                            request_id: rid,
                            response,
                        },
                    ..
                },
            )) => {
                if request_id == Some(rid) {
                    let (env, _xfer) = crate::p2p::transfer::parse_xfer_response(&response)?;
                    if env.request_id != expected_req_id {
                        return Err(FossilP2pError::Protocol(
                            "response request_id mismatch".into(),
                        ));
                    }
                    return Ok(response);
                }
            }
            SwarmEvent::Behaviour(FossilP2pBehaviourEvent::Xfer(
                request_response::Event::OutboundFailure {
                    request_id: rid,
                    error,
                    ..
                },
            )) => {
                if request_id == Some(rid) {
                    return Err(FossilP2pError::P2p(format!("outbound failure: {error}")));
                }
            }
            SwarmEvent::ConnectionClosed { peer_id: p, .. } if p == peer_id && connected => {
                return Err(FossilP2pError::P2p(
                    "connection closed before response".into(),
                ));
            }
            _ => {}
        }
    }
}

/// Receiver side: validate an inbound authenticated transfer request and dispatch it.
pub fn handle_inbound_xfer(
    envelope_bytes: &[u8],
    home: &Home,
    config: &FossilP2pConfig,
    keypair: &Keypair,
) -> Vec<u8> {
    let parsed = match parse_xfer_envelope(envelope_bytes, None) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("  xfer rejected: {e}");
            return Vec::new();
        }
    };

    let rid = &parsed.xfer.rid;
    let operation = parsed.xfer.operation;
    let request_id = parsed.env.request_id;
    let requester = parsed.env.sender;

    let db = match Database::open(&home.db()) {
        Ok(db) => db,
        Err(e) => {
            eprintln!("  xfer storage error: {e}");
            return build_xfer_error(keypair, request_id, rid, "local storage unavailable")
                .unwrap_or_default();
        }
    };

    let record = match db.load_repository(rid) {
        Ok(Some(r)) => r,
        Ok(None) => {
            eprintln!("  xfer: unknown repository {rid}");
            return build_xfer_error(keypair, request_id, rid, "repository not found locally")
                .unwrap_or_default();
        }
        Err(e) => {
            eprintln!("  xfer db error: {e}");
            return build_xfer_error(keypair, request_id, rid, "storage error").unwrap_or_default();
        }
    };

    if !check_peer_authorized(&record.owner_key, &record.visibility, &requester, operation) {
        eprintln!(
            "  xfer: peer {} not authorized for {rid} ({:?})",
            requester.to_multibase(),
            operation
        );
        return build_xfer_error(keypair, request_id, rid, "not authorized").unwrap_or_default();
    }

    // `Info` is a metadata-only operation: return the signed advertisement without
    // touching the Fossil repository. Used by `repo clone` for identity handoff.
    if operation == Operation::Info {
        return match build_info_response(keypair, request_id, rid, home, &record) {
            Ok(bytes) => bytes,
            Err(e) => build_xfer_error(keypair, request_id, rid, &e.to_string()).unwrap_or_default(),
        };
    }

    let repo_path = Path::new(&record.path);
    if !repo_path.exists() {
        return build_xfer_error(keypair, request_id, rid, "repository path missing")
            .unwrap_or_default();
    }

    match run_receiver_request(&parsed.xfer.payload, repo_path, &config.fossil.fossil_path) {
        Ok(response) => {
            if let Ok(pid) = peer_id_from_public_key(&requester) {
                let _ = db.update_peer_seen(&pid.to_string());
            }
            build_xfer_response(keypair, request_id, rid, response)
                .unwrap_or_else(|e| {
                    eprintln!("  xfer response signing failed: {e}");
                    Vec::new()
                })
        }
        Err(e) => {
            eprintln!("  fossil test-http failed: {e}");
            build_xfer_error(keypair, request_id, rid, &e.to_string()).unwrap_or_default()
        }
    }
}

fn peer_id_from_public_key(pk: &PublicKey) -> Result<PeerId> {
    Ok(pk.to_libp2p_peer_id())
}

/// Build an `Info` response carrying the signed repository advertisement.
/// The advertisement is retrieved from local storage where it was persisted,
/// signed by the repository owner.
fn build_info_response(
    keypair: &Keypair,
    request_id: [u8; 16],
    rid: &str,
    home: &Home,
    record: &crate::storage::RepositoryRecord,
) -> Result<Vec<u8>> {
    let db = Database::open(&home.db())?;

    let stored = db.find_repo_providers(rid)?.into_iter().find_map(|r| r.advertisement_json);

    let payload = match stored {
        Some(json) => {
            let bytes = json.into_bytes();
            // Validate before serving.
            let ad = crate::discovery::parse_advertisement(&bytes)?;
            if !ad.verify() {
                return Err(FossilP2pError::Protocol(
                    "stored advertisement signature invalid".into(),
                ));
            }
            bytes
        }
        None => {
            // Fall back to a freshly signed advertisement from the local owner key.
            let ad = crate::protocol::RepoAdvertisement::new(
                rid,
                keypair,
                &keypair.public_key().to_did_key(),
                record.creation_nonce.as_deref().unwrap_or(""),
                &record.name,
                record.description.as_deref().unwrap_or(""),
                &record.visibility.as_db_str(),
                vec![],
            );
            crate::protocol::encode_advertisement(&ad)?
        }
    };

    build_xfer_response(keypair, request_id, rid, payload)
}

/// Run a raw Fossil HTTP request against a repository directory.
pub fn run_receiver_request(
    request_bytes: &[u8],
    repo_path: &Path,
    fossil_path: &str,
) -> Result<Vec<u8>> {
    let tmp_dir = std::env::temp_dir().join("peergit-xfer");
    fs::create_dir_all(&tmp_dir)?;

    let req_id = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let request_file = tmp_dir.join(format!("req-{req_id}.txt"));

    fs::write(&request_file, request_bytes)?;

    let output = std::process::Command::new(fossil_path)
        .args(["test-http", &request_file.to_string_lossy()])
        .current_dir(repo_path)
        .output()
        .map_err(|e| FossilP2pError::Fossil(format!("failed to run fossil test-http: {e}")))?;

    let _ = fs::remove_file(&request_file);

    if output.status.success() {
        Ok(output.stdout)
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(FossilP2pError::Fossil(format!(
            "fossil test-http failed: {stderr}"
        )))
    }
}

/// Resolve a peer string (multiaddr, peer id, or alias) to a (PeerId, Multiaddr).
/// Resolve the peer/address for a parsed transport target, falling back to RID
/// discovery when the URL names no specific peer.
pub fn resolve_target(
    home: &Home,
    config: &FossilP2pConfig,
    keypair: &libp2p::identity::Keypair,
    target: &TransportTarget,
) -> Result<(PeerId, Multiaddr)> {
    let peer = target.peer.trim();
    if !peer.is_empty() && !RESOLVE_HOSTS.contains(&peer) {
        return resolve_peer(home, peer);
    }
    if target.rid.is_empty() {
        return Err(FossilP2pError::Protocol(
            "transport URL names neither a peer nor a RID".into(),
        ));
    }
    resolve_provider_for_rid(home, config, keypair, &target.rid)
}

/// Look up a reachable provider for `rid` from the local cache, then the DHT.
pub fn resolve_provider_for_rid(
    home: &Home,
    config: &FossilP2pConfig,
    keypair: &libp2p::identity::Keypair,
    rid: &str,
) -> Result<(PeerId, Multiaddr)> {
    let db = Database::open(&home.db())?;
    for record in db.find_repo_providers(rid)? {
        let peer_id = match record.peer_id.parse::<PeerId>() {
            Ok(id) => id,
            Err(_) => continue,
        };
        if let Ok(Some(peer)) = db.get_peer(&record.peer_id) {
            if let Some(addrs) = peer.addresses {
                for addr_str in addrs.split(',') {
                    let addr_str = addr_str.trim();
                    if addr_str.is_empty() {
                        continue;
                    }
                    if let Ok(address) = addr_str.parse::<Multiaddr>() {
                        return Ok((peer_id, ensure_peer_suffix(address, peer_id)));
                    }
                }
            }
        }
    }

    let providers = tokio::runtime::Runtime::new()
        .map_err(|e| FossilP2pError::P2p(format!("tokio runtime: {e}")))?
        .block_on(async {
            crate::discovery::lookup_repo_providers(
                &config.p2p,
                keypair,
                rid,
                Duration::from_secs(25),
            )
            .await
        })?;

    for provider in providers {
        if let Some(first) = provider.addresses.first() {
            if let Ok(address) = first.parse::<Multiaddr>() {
                return Ok((
                    provider.peer_id,
                    ensure_peer_suffix(address, provider.peer_id),
                ));
            }
        }
    }

    Err(FossilP2pError::P2p(format!(
        "no reachable provider found for {rid}"
    )))
}

/// Append `/p2p/<peer_id>` to a multiaddr when it is missing.
pub fn ensure_peer_suffix(mut addr: Multiaddr, peer_id: PeerId) -> Multiaddr {
    let has_peer = addr
        .iter()
        .any(|p| matches!(p, libp2p::multiaddr::Protocol::P2p(_)));
    if !has_peer {
        addr.push(libp2p::multiaddr::Protocol::P2p(peer_id));
    }
    addr
}

pub fn resolve_peer(home: &Home, peer: &str) -> Result<(PeerId, Multiaddr)> {
    if let Ok(addr) = peer.parse::<Multiaddr>() {
        let peer_id = extract_peer_id_from_multiaddr(&addr)?;
        return Ok((peer_id, addr));
    }

    let db = Database::open(&home.db())
        .map_err(|_| FossilP2pError::P2p("cannot open database for peer lookup".into()))?;

    let peers = db
        .list_peers()
        .map_err(|e| FossilP2pError::P2p(format!("failed to list peers: {e}")))?;

    for record in &peers {
        if record.peer_id == peer || record.alias.as_deref() == Some(peer) {
            if let Some(addrs) = &record.addresses {
                for addr_str in addrs.split(',') {
                    if let Ok(addr) = addr_str.trim().parse::<Multiaddr>() {
                        if let Ok(peer_id) = extract_peer_id_from_multiaddr(&addr) {
                            if peer_id.to_string() == record.peer_id {
                                return Ok((peer_id, addr));
                            }
                        }
                    }
                }
            }
            return Err(FossilP2pError::P2p(format!(
                "peer {peer} found but no address known. Add with: peergit peer add <key> --addresses <multiaddr>"
            )));
        }
    }

    Err(FossilP2pError::P2p(format!(
        "cannot resolve peer: {peer}. Use a multiaddr or add the peer first."
    )))
}

/// Open a one-shot connection to a peer and request the signed advertisement (`Info`).
pub async fn fetch_advertisement(
    config: &FossilP2pConfig,
    app_keypair: &Keypair,
    libp2p_keypair: &libp2p::identity::Keypair,
    peer_id: PeerId,
    rid: &str,
) -> Result<Option<crate::protocol::RepoAdvertisement>> {
    let (envelope, _req_id) = build_xfer_request(app_keypair, rid, Operation::Info, Vec::new())?;

    let mut swarm = build_oneshot_swarm(config, libp2p_keypair)?;
    swarm
        .dial(peer_id)
        .map_err(|e| FossilP2pError::P2p(format!("dial failed: {e}")))?;

    let response = tokio::time::timeout(Duration::from_secs(60), async {
        let mut connected = false;
        let mut request_id: Option<request_response::OutboundRequestId> = None;
        loop {
            match swarm.select_next_some().await {
                SwarmEvent::ConnectionEstablished { peer_id: p, .. } if p == peer_id => {
                    let id = swarm
                        .behaviour_mut()
                        .xfer
                        .send_request(&peer_id, envelope.clone());
                    request_id = Some(id);
                    connected = true;
                }
                SwarmEvent::Behaviour(FossilP2pBehaviourEvent::Xfer(
                    request_response::Event::Message {
                        message:
                            request_response::Message::Response {
                                request_id: rid,
                                response,
                            },
                        ..
                    },
                )) => {
                    if request_id == Some(rid) {
                        return Ok::<Vec<u8>, FossilP2pError>(response);
                    }
                }
                SwarmEvent::Behaviour(FossilP2pBehaviourEvent::Xfer(
                    request_response::Event::OutboundFailure {
                        request_id: rid,
                        error,
                        ..
                    },
                )) => {
                    if request_id == Some(rid) {
                        return Err(FossilP2pError::P2p(format!("outbound failure: {error}")));
                    }
                }
                SwarmEvent::ConnectionClosed { peer_id: p, .. } if p == peer_id && connected => {
                    return Err(FossilP2pError::P2p(
                        "connection closed before Info response".into(),
                    ));
                }
                _ => {}
            }
        }
    })
    .await
    .map_err(|_| FossilP2pError::P2p("timeout waiting for advertisement".into()))??;

    let (_env, xfer) = crate::p2p::transfer::parse_xfer_response(&response)?;
    if !xfer.ok {
        return Err(FossilP2pError::P2p(format!(
            "remote rejected Info request: {}",
            xfer.error.unwrap_or_default()
        )));
    }
    if xfer.payload.is_empty() {
        return Ok(None);
    }
    Ok(Some(crate::discovery::parse_advertisement(&xfer.payload)?))
}

fn extract_peer_id_from_multiaddr(addr: &Multiaddr) -> Result<PeerId> {
    for protocol in addr.iter() {
        if let libp2p::multiaddr::Protocol::P2p(peer_id) = protocol {
            return Ok(peer_id);
        }
    }
    Err(FossilP2pError::P2p(
        "multiaddr does not contain a /p2p/ component".into(),
    ))
}

fn build_oneshot_swarm(
    config: &FossilP2pConfig,
    keypair: &libp2p::identity::Keypair,
) -> Result<Swarm<FossilP2pBehaviour>> {
    crate::p2p::transport::build_oneshot_swarm(&config.p2p, keypair)
}

pub fn load_keypair(home: &Home) -> Result<Keypair> {
    let sk_path = home.secret_key_path();
    let sk_hex = fs::read_to_string(&sk_path).map_err(|e| {
        FossilP2pError::P2p(format!(
            "failed to read secret key at {}: {e}",
            sk_path.display()
        ))
    })?;
    let sk_bytes = hex::decode(sk_hex.trim())
        .map_err(|e| FossilP2pError::Crypto(format!("invalid key hex: {e}")))?;
    let sk_arr: [u8; 32] = sk_bytes
        .try_into()
        .map_err(|_| FossilP2pError::Crypto("invalid secret key length".into()))?;
    Keypair::from_bytes(&sk_arr)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_url_with_peer_and_rid() {
        let t = parse_transport_url("peergit://12D3KooWabc/rid123").unwrap();
        assert_eq!(t.peer, "12D3KooWabc");
        assert_eq!(t.rid, "rid123");
    }

    #[test]
    fn parse_url_rid_only() {
        let t = parse_transport_url("peergit://rid123").unwrap();
        assert_eq!(t.peer, "");
        assert_eq!(t.rid, "rid123");
    }

    #[test]
    fn parse_url_bad_scheme() {
        assert!(parse_transport_url("ftp://example.com").is_err());
    }

    #[test]
    fn parse_url_http_resolve_host() {
        let rid = "c".repeat(64);
        let t = parse_transport_url(&format!("http://peergit/{rid}")).unwrap();
        assert_eq!(t.peer, "peergit");
        assert_eq!(t.rid, rid);
    }

    #[test]
    fn parse_url_http_xfer_segment() {
        let rid = "d".repeat(64);
        let t = parse_transport_url(&format!("http://peergit/{rid}/xfer?x=1")).unwrap();
        assert_eq!(t.peer, "peergit");
        assert_eq!(t.rid, rid);
    }

    #[test]
    fn parse_url_multiaddr_with_rid() {
        let rid = "a".repeat(64);
        let url = format!(
            "peergit:///ip4/127.0.0.1/tcp/4001/p2p/12D3KooWabc/{rid}"
        );
        let t = parse_transport_url(&url).unwrap();
        assert_eq!(
            t.peer,
            "/ip4/127.0.0.1/tcp/4001/p2p/12D3KooWabc"
        );
        assert_eq!(t.rid, rid);
    }

    #[test]
    fn parse_url_strips_fossil_method_segment() {
        let rid = "b".repeat(64);
        let url = format!("peergit:///ip4/127.0.0.1/tcp/4001/p2p/12D3KooWabc/{rid}/xfer");
        let t = parse_transport_url(&url).unwrap();
        assert_eq!(t.rid, rid);
    }

    #[test]
    fn parse_url_multiaddr_without_rid() {
        let t =
            parse_transport_url("peergit:///ip4/127.0.0.1/tcp/4001/p2p/12D3KooWabc").unwrap();
        assert_eq!(t.rid, "");
        assert_eq!(t.peer, "/ip4/127.0.0.1/tcp/4001/p2p/12D3KooWabc");
    }
}