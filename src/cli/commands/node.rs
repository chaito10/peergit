use crate::error::{FossilP2pError, Result};
use crate::p2p::behaviour::FossilP2pBehaviourEvent;
use crate::storage::Database;

use super::util::{get_config, get_home};

pub fn cmd_node_start() -> Result<()> {
    let home = get_home()?;
    home.init()?;
    let config = get_config(&home)?;
    let keypair = crate::keystore::load_or_create_keypair(&home)?;
    let pk = keypair.public_key();

    println!("Starting PeerGit node...");
    println!("  Alias:     {}", config.node.alias);
    println!("  Peer ID:   {}", pk.to_libp2p_peer_id());
    println!("  Listening: {}", config.p2p.listen.join(", "));
    println!("  Kademlia:  {}", config.p2p.kad_protocol);
    println!("  Web UI:    http://localhost:{}", config.fossil.web_port);
    println!("  Log:       {}", config.node.log);

    tokio::runtime::Runtime::new()?.block_on(async move {
        let libp2p_keypair = keypair
            .to_libp2p_keypair()
            .map_err(|e| FossilP2pError::P2p(format!("key conversion: {e}")))?;

        let mut swarm = crate::p2p::transport::build_swarm(&config.p2p, &libp2p_keypair)?;

        for peer_str in &config.p2p.bootstrap_peers {
            if let Ok(multiaddr) = peer_str.parse::<libp2p::Multiaddr>() {
                let peer_id_opt = multiaddr.iter().find_map(|p| match p {
                    libp2p::multiaddr::Protocol::P2p(id) => Some(id),
                    _ => None,
                });
                if let Some(peer_id) = peer_id_opt {
                    swarm
                        .behaviour_mut()
                        .kad
                        .add_address(&peer_id, multiaddr.clone());
                    println!("  Bootstrap: {peer_str}");
                }
            }
        }

        let web_state = std::sync::Arc::new(crate::web::WebState {
            home: home.clone(),
            config: config.clone(),
        });
        let web_port = config.fossil.web_port;
        tokio::spawn(async move {
            if let Err(e) = crate::web::start_web_server(web_state, web_port).await {
                eprintln!("web server error: {e}");
            }
        });

        use futures::StreamExt;
        use libp2p::request_response;
        use libp2p::swarm::SwarmEvent;

        println!("\nNode running. Press Ctrl+C to stop.\n");

        let mut collected_addrs: Vec<String> = Vec::new();
        let mut advertised = false;

        loop {
            match swarm.select_next_some().await {
                SwarmEvent::NewListenAddr { address, .. } => {
                    println!("  Listening on: {address}");
                    collected_addrs.push(address.to_string());

                    if !advertised {
                        let db = match Database::open(&home.db()) {
                            Ok(db) => db,
                            Err(e) => {
                                eprintln!("  db error: {e}");
                                continue;
                            }
                        };
                        match crate::discovery::advertise_local_repos(
                            &mut swarm,
                            &keypair,
                            &db,
                            collected_addrs.clone(),
                        ) {
                            Ok(n) if n > 0 => {
                                println!("  Advertised {n} repository/repositories to the DHT");
                                advertised = true;
                            }
                            Ok(_) => {}
                            Err(e) => eprintln!("  advertisement error: {e}"),
                        }
                    }
                }
                SwarmEvent::Behaviour(FossilP2pBehaviourEvent::Identify(
                    libp2p::identify::Event::Received { peer_id, info, .. },
                )) => {
                    println!("  Identified: {peer_id}");
                    for addr in &info.listen_addrs {
                        swarm
                            .behaviour_mut()
                            .kad
                            .add_address(&peer_id, addr.clone());
                    }

                    if let Ok(db) = Database::open(&home.db()) {
                        let pk_hex = match info.public_key.clone().try_into_ed25519() {
                            Ok(ed) => hex::encode(ed.to_bytes()),
                            Err(_) => peer_id.to_string(),
                        };

                        let _ = db.store_peer(
                            &peer_id.to_string(),
                            &pk_hex,
                            Some(info.agent_version.as_str()),
                            Some(
                                &info
                                    .listen_addrs
                                    .iter()
                                    .map(|a| a.to_string())
                                    .collect::<Vec<_>>()
                                    .join(","),
                            ),
                        );

                        let _ = crate::discovery::advertise_local_repos(
                            &mut swarm,
                            &keypair,
                            &db,
                            collected_addrs.clone(),
                        );
                    }
                }
                SwarmEvent::Behaviour(FossilP2pBehaviourEvent::Kad(
                    libp2p::kad::Event::RoutingUpdated { peer, .. },
                )) => {
                    tracing::debug!("DHT routing updated: {peer}");
                    println!("  DHT routing updated: {peer}");
                }
                SwarmEvent::Behaviour(FossilP2pBehaviourEvent::Kad(
                    libp2p::kad::Event::OutboundQueryProgressed { .. },
                )) => {
                    // DHT queries progress; providers are logged at result time.
                }
                SwarmEvent::Behaviour(FossilP2pBehaviourEvent::Ping(
                    libp2p::ping::Event {
                        peer,
                        result: Ok(rtt),
                        ..
                    },
                )) => {
                    tracing::debug!("Ping {peer}: {rtt:?}");
                }
                SwarmEvent::Behaviour(FossilP2pBehaviourEvent::Xfer(
                    request_response::Event::Message {
                        peer,
                        message:
                            request_response::Message::Request {
                                request_id,
                                request,
                                channel,
                            },
                        ..
                    },
                )) => {
                    println!("  Xfer request from {peer} ({request_id})");
                    let response = crate::transport::handle_inbound_xfer(
                        &request,
                        &home,
                        &config,
                        &keypair,
                    );
                    let _ = swarm.behaviour_mut().xfer.send_response(channel, response);
                }
                SwarmEvent::Behaviour(FossilP2pBehaviourEvent::Xfer(
                    request_response::Event::OutboundFailure { peer, error, .. },
                )) => {
                    println!("  Xfer outbound error to {peer}: {error}");
                }
                _ => {}
            }
        }
    })
}

pub fn cmd_node_status() -> Result<()> {
    let home = get_home()?;
    let keypair = crate::keystore::load_or_create_keypair(&home)?;
    let pk = keypair.public_key();
    let config = get_config(&home)?;

    println!("Node Status:");
    println!("  Alias:      {}", config.node.alias);
    println!("  Peer ID:    {}", pk.to_libp2p_peer_id());
    println!("  Public Key: {}", pk);
    println!("  Listening:  {}", config.p2p.listen.join(", "));
    println!("  Log Level:  {}", config.node.log);
    Ok(())
}