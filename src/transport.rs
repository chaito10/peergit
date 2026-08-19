use crate::config::FossilP2pConfig;
use crate::crypto::Keypair;
use crate::error::{FossilP2pError, Result};
use crate::home::Home;
use crate::p2p::behaviour::{FossilP2pBehaviour, FossilP2pBehaviourEvent};
use crate::storage::Database;
use futures::StreamExt;
use libp2p::{Multiaddr, PeerId, Swarm, SwarmBuilder, request_response, swarm::SwarmEvent};
use std::fs;
use std::path::Path;
use std::time::{Duration, Instant};

pub fn run_transport(url: &str, request_file: &Path, reply_file: &Path) -> Result<()> {
    let request = fs::read(request_file).map_err(|e| {
        FossilP2pError::P2p(format!(
            "failed to read request file {}: {e}",
            request_file.display()
        ))
    })?;

    let (peer_id, address) = parse_url(url)?;

    let home = Home::new()?;
    let config = FossilP2pConfig::load(&home.config())?;
    let keypair = load_keypair(&home)?;
    let libp2p_keypair = keypair
        .to_libp2p_keypair()
        .map_err(|e| FossilP2pError::P2p(format!("key conversion: {e}")))?;

    let mut swarm = build_oneshot_swarm(&config, &libp2p_keypair)?;

    swarm
        .dial(address)
        .map_err(|e| FossilP2pError::P2p(format!("dial failed: {e}")))?;

    let response = tokio::runtime::Runtime::new()
        .map_err(|e| FossilP2pError::P2p(format!("tokio runtime: {e}")))?
        .block_on(async move {
            let mut connected = false;
            let mut request_id: Option<request_response::OutboundRequestId> = None;
            let deadline = Instant::now() + Duration::from_secs(30);

            loop {
                if Instant::now() > deadline {
                    return Err(FossilP2pError::P2p("timeout waiting for response".into()));
                }

                match swarm.select_next_some().await {
                    SwarmEvent::ConnectionEstablished {
                        peer_id: p, ..
                    } if p == peer_id => {
                        tracing::info!("connected to {peer_id}");
                        let id = swarm
                            .behaviour_mut()
                            .xfer
                            .send_request(&peer_id, request.clone());
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
                            return Err(FossilP2pError::P2p(format!(
                                "outbound failure: {error}"
                            )));
                        }
                    }
                    SwarmEvent::ConnectionClosed {
                        peer_id: p, ..
                    } if p == peer_id && connected => {
                        return Err(FossilP2pError::P2p(
                            "connection closed before response".into(),
                        ));
                    }
                    _ => {}
                }
            }
        })?;

    fs::write(reply_file, &response).map_err(|e| {
        FossilP2pError::P2p(format!(
            "failed to write reply file {}: {e}",
            reply_file.display()
        ))
    })?;

    Ok(())
}

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

fn parse_url(url: &str) -> Result<(PeerId, Multiaddr)> {
    if let Ok(addr) = url.parse::<Multiaddr>() {
        let peer_id = extract_peer_id_from_multiaddr(&addr)?;
        return Ok((peer_id, addr));
    }

    let db = Home::new()
        .and_then(|home| Database::open(&home.db()))
        .map_err(|_| FossilP2pError::P2p("cannot open database for peer lookup".into()))?;

    let peers = db.list_peers().map_err(|e| {
        FossilP2pError::P2p(format!("failed to list peers: {e}"))
    })?;

    for (pid, _pk, alias, addresses, _last_seen) in &peers {
        if pid == url || alias.as_deref() == Some(url) {
            if let Some(addrs) = addresses {
                for addr_str in addrs.split(',') {
                    if let Ok(addr) = addr_str.trim().parse::<Multiaddr>() {
                        if let Ok(peer_id) = extract_peer_id_from_multiaddr(&addr) {
                            if peer_id.to_string() == *pid {
                                return Ok((peer_id, addr));
                            }
                        }
                    }
                }
            }
            let _peer_id: PeerId = pid.parse().map_err(|_| {
                FossilP2pError::P2p(format!("invalid peer ID in database: {pid}"))
            })?;
            return Err(FossilP2pError::P2p(format!(
                "peer {url} found but no address known. Add with: peergit peer add <key> --addresses <multiaddr>"
            )));
        }
    }

    Err(FossilP2pError::P2p(format!(
        "cannot resolve peer: {url}. Use a multiaddr or add the peer first."
    )))
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
    let kad_protocol = config.p2p.kad_protocol.clone();
    let idle_timeout = Duration::from_secs(config.p2p.idle_timeout_secs);

    let mut swarm = SwarmBuilder::with_existing_identity(keypair.clone())
        .with_tokio()
        .with_tcp(
            libp2p::tcp::Config::default(),
            libp2p::noise::Config::new,
            libp2p::yamux::Config::default,
        )
        .map_err(|e| FossilP2pError::P2p(format!("tcp setup: {e}")))?
        .with_behaviour(|key| {
            let peer_id = key.public().to_peer_id();
            Ok(FossilP2pBehaviour::new(peer_id, key, &kad_protocol))
        })
        .map_err(|e| FossilP2pError::P2p(format!("behaviour setup: {e}")))?
        .with_swarm_config(|cfg| cfg.with_idle_connection_timeout(idle_timeout))
        .build();

    if let Some(addr_str) = config.p2p.listen.first() {
        if let Ok(addr) = addr_str.parse() {
            let _ = swarm.listen_on(addr);
        }
    }

    Ok(swarm)
}

fn load_keypair(home: &Home) -> Result<Keypair> {
    let sk_path = home.secret_key_path();
    let sk_hex = fs::read_to_string(&sk_path).map_err(|e| {
        FossilP2pError::P2p(format!(
            "failed to read secret key at {}: {e}",
            sk_path.display()
        ))
    })?;
    let sk_bytes = hex::decode(sk_hex.trim())
        .map_err(|e| FossilP2pError::Crypto(format!("invalid key hex: {e}")))?;
    let sk_arr: [u8; 32] = sk_bytes.try_into().map_err(|_| {
        FossilP2pError::Crypto("invalid secret key length".into())
    })?;
    Keypair::from_bytes(&sk_arr)
}
