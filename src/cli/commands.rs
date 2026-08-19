use crate::config::FossilP2pConfig;
use crate::crypto::{Keypair, PublicKey};
use crate::error::Result;
use crate::fossil::FossilCli;
use crate::home::Home;
use crate::repository::FossilRepoManager;
use crate::storage::Database;
use clap::{Parser, Subcommand};
use std::fs;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "peergit")]
#[command(version = "0.1.0")]
#[command(about = "P2P transport, discovery, identity and collaboration layer for Fossil repositories")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    Init,
    Identity,
    Peer {
        #[command(subcommand)]
        command: PeerCommands,
    },
    Node {
        #[command(subcommand)]
        command: NodeCommands,
    },
    Repo {
        #[command(subcommand)]
        command: RepoCommands,
    },
    Sync {
        #[arg(short, long)]
        rid: Option<String>,
    },
    Config {
        #[command(subcommand)]
        command: Option<ConfigCommands>,
    },
    Fossil {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    Transport {
        url: String,
        request_file: PathBuf,
        reply_file: PathBuf,
    },
}

#[derive(Subcommand)]
pub enum PeerCommands {
    Add {
        public_key: String,
        #[arg(short, long)]
        alias: Option<String>,
        #[arg(short, long)]
        addresses: Vec<String>,
    },
    List,
}

#[derive(Subcommand)]
pub enum NodeCommands {
    Start,
    Status,
}

#[derive(Subcommand)]
pub enum RepoCommands {
    List,
    Publish {
        #[arg(short, long)]
        path: Option<PathBuf>,
        #[arg(short, long)]
        name: Option<String>,
        #[arg(short, long)]
        description: Option<String>,
    },
    Unpublish {
        rid: String,
    },
    Discover {
        rid: String,
    },
    Clone {
        rid: String,
        #[arg(default_value = ".")]
        directory: PathBuf,
    },
}

#[derive(Subcommand)]
pub enum ConfigCommands {
    Show,
    Init,
    Get {
        key: String,
    },
    Set {
        key: String,
        value: String,
    },
}

fn get_home() -> Result<Home> {
    Home::new()
}

fn get_db(home: &Home) -> Result<Database> {
    Database::open(&home.db())
}

fn get_keypair(home: &Home) -> Result<Keypair> {
    let sk_path = home.secret_key_path();
    if sk_path.exists() {
        let sk_hex = fs::read_to_string(&sk_path)?;
        let sk_bytes = hex::decode(sk_hex.trim())
            .map_err(|e| crate::error::FossilP2pError::Crypto(format!("invalid key hex: {e}")))?;
        let sk_arr: [u8; 32] = sk_bytes
            .try_into()
            .map_err(|_| crate::error::FossilP2pError::Crypto("invalid secret key length".into()))?;
        Keypair::from_bytes(&sk_arr)
    } else {
        let keypair = Keypair::generate();
        fs::create_dir_all(home.keys())?;
        fs::write(&sk_path, hex::encode(keypair.secret_bytes()))?;
        fs::write(home.public_key_path(), hex::encode(keypair.public_key().to_bytes()))?;
        Ok(keypair)
    }
}

fn get_config(home: &Home) -> Result<FossilP2pConfig> {
    FossilP2pConfig::load(&home.config())
}

fn cmd_init() -> Result<()> {
    let home = get_home()?;
    home.init()?;
    let keypair = get_keypair(&home)?;
    let pk = keypair.public_key();
    let did = crate::identity::Did::from_public_key(&pk);

    let db = get_db(&home)?;
    db.store_identity(&pk.to_hex(), &did.to_string(), None)?;

    println!("Node initialized!");
    println!("  Public Key: {}", pk);
    println!("  DID:        {}", did);
    println!("  Peer ID:    {}", pk.to_libp2p_peer_id());
    println!("  Home:       {}", home.path.display());
    Ok(())
}

fn cmd_identity() -> Result<()> {
    let home = get_home()?;
    let keypair = get_keypair(&home)?;
    let pk = keypair.public_key();
    let did = crate::identity::Did::from_public_key(&pk);

    println!("Node Identity:");
    println!("  Public Key: {}", pk);
    println!("  DID:        {}", did);
    println!("  Peer ID:    {}", pk.to_libp2p_peer_id());
    println!("  Key Path:   {}", home.secret_key_path().display());
    Ok(())
}

fn cmd_peer_add(
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

fn cmd_peer_list() -> Result<()> {
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
    for (peer_id, pk, alias, _addresses, last_seen) in &peers {
        let alias_str = alias.as_deref().unwrap_or("-");
        let pk_short = if pk.len() > 24 {
            format!("{}...", &pk[..21])
        } else {
            pk.clone()
        };
        let last_short = if last_seen.len() > 10 {
            &last_seen[..10]
        } else {
            last_seen
        };
        println!(
            "{:<65} {:<25} {:<20} {:<30}",
            peer_id, pk_short, alias_str, last_short
        );
    }
    Ok(())
}

fn cmd_node_start() -> Result<()> {
    let home = get_home()?;
    let config = get_config(&home)?;
    let keypair = get_keypair(&home)?;
    let pk = keypair.public_key();

    println!("Starting PeerGit node...");
    println!("  Alias:     {}", config.node.alias);
    println!("  Peer ID:   {}", pk.to_libp2p_peer_id());
    println!("  Listening: {}", config.p2p.listen.join(", "));
    println!("  Kademlia:  {}", config.p2p.kad_protocol);
    println!("  Web UI:    http://localhost:{}", config.fossil.web_port);
    println!("  Log:       {}", config.node.log);

    tokio::runtime::Runtime::new()?.block_on(async {
        let libp2p_keypair = keypair.to_libp2p_keypair()
            .map_err(|e| crate::error::FossilP2pError::P2p(format!("key conversion: {e}")))?;

        let mut swarm = crate::p2p::transport::build_swarm(&config.p2p, &libp2p_keypair)?;

        for peer_str in &config.p2p.bootstrap_peers {
            if let Ok(multiaddr) = peer_str.parse::<libp2p::Multiaddr>() {
                let peer_id_opt = multiaddr.iter().find_map(|p| match p {
                    libp2p::multiaddr::Protocol::P2p(id) => Some(id),
                    _ => None,
                });
                if let Some(peer_id) = peer_id_opt {
                    swarm.behaviour_mut().kad.add_address(&peer_id, multiaddr.clone());
                    println!("  Bootstrap: {}", peer_str);
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
        use crate::p2p::behaviour::FossilP2pBehaviourEvent;

        println!("\nNode running. Press Ctrl+C to stop.\n");

        loop {
            match swarm.select_next_some().await {
                SwarmEvent::NewListenAddr { address, .. } => {
                    println!("  Listening on: {address}");
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
                }
                SwarmEvent::Behaviour(FossilP2pBehaviourEvent::Kad(
                    libp2p::kad::Event::RoutingUpdated { peer, .. },
                )) => {
                    println!("  DHT routing updated: {peer}");
                }
                SwarmEvent::Behaviour(FossilP2pBehaviourEvent::Ping(
                    libp2p::ping::Event {
                        peer,
                        result: Ok(rtt),
                        ..
                    },
                )) => {
                    println!("  Ping {peer}: {rtt:?}");
                }
                SwarmEvent::Behaviour(FossilP2pBehaviourEvent::Xfer(
                    request_response::Event::Message {
                        peer,
                        message: request_response::Message::Request {
                            request_id,
                            request,
                            channel,
                        },
                        ..
                    },
                )) => {
                    println!("  Xfer request from {peer} ({request_id})");
                    let fossil_path = config.fossil.fossil_path.clone();
                    let repos = crate::storage::Database::open(&home.db())
                        .ok()
                        .and_then(|db| db.list_repositories().ok())
                        .unwrap_or_default();

                    let mut response = Vec::new();
                    for (_rid, _name, _desc, _path) in &repos {
                        let repo_path = std::path::Path::new(_path);
                        if repo_path.exists() {
                            match crate::transport::run_receiver_request(
                                &request, repo_path, &fossil_path,
                            ) {
                                Ok(resp) => { response = resp; break; }
                                Err(e) => {
                                    eprintln!("  xfer error: {e}");
                                }
                            }
                        }
                    }

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

fn cmd_node_status() -> Result<()> {
    let home = get_home()?;
    let keypair = get_keypair(&home)?;
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

fn cmd_repo_list() -> Result<()> {
    let home = get_home()?;
    let db = get_db(&home)?;
    let repos = db.list_repositories()?;
    if repos.is_empty() {
        println!("No published repositories.");
        return Ok(());
    }
    println!(
        "{:<66} {:<30} {:<12}",
        "RID", "NAME", "VISIBILITY"
    );
    println!("{}", "-".repeat(110));
    for (rid, name, _desc, vis) in &repos {
        let rid_short = if rid.len() > 64 {
            format!("{}...", &rid[..61])
        } else {
            rid.clone()
        };
        let name_short = if name.len() > 29 {
            format!("{}...", &name[..26])
        } else {
            name.clone()
        };
        println!("{:<66} {:<30} {:<12}", rid_short, name_short, vis);
    }
    Ok(())
}

fn cmd_repo_publish(
    path: Option<PathBuf>,
    name: Option<String>,
    description: Option<String>,
) -> Result<()> {
    let home = get_home()?;
    home.init()?;
    let keypair = get_keypair(&home)?;
    let pk = keypair.public_key();
    let db = get_db(&home)?;

    let repo_path = path.unwrap_or_else(|| std::env::current_dir().unwrap_or_default());
    let repo_name = name
        .or_else(|| {
            repo_path
                .file_name()
                .and_then(|n| n.to_str())
                .map(|s| s.to_string())
        })
        .unwrap_or_else(|| "my-project".to_string());
    let desc = description.unwrap_or_default();

    let fossil = FossilCli::new(&FossilP2pConfig::default().fossil);
    let manager = FossilRepoManager::new(fossil);
    let repo_identity = manager.init_repo(&repo_path, &repo_name, &desc, &pk)?;

    db.store_repository(
        &repo_identity.rid,
        &repo_identity.name,
        Some(&repo_identity.description),
        &repo_path.to_string_lossy(),
        &pk.to_hex(),
        "public",
        None,
    )?;

    println!("Repository published!");
    println!("  RID:      {}", repo_identity.rid);
    println!("  Name:     {}", repo_identity.name);
    println!("  Owner:    {}", repo_identity.owner_did);
    println!("  Path:     {}", repo_path.display());
    Ok(())
}

fn cmd_repo_discover(rid: String) -> Result<()> {
    let home = get_home()?;
    let db = get_db(&home)?;
    match db.load_repository(&rid)? {
        Some((name, desc, path, _owner, _vis, _fossil_db)) => {
            println!("Repository found:");
            println!("  RID:         {}", rid);
            println!("  Name:        {}", name);
            if let Some(d) = desc {
                println!("  Description: {}", d);
            }
            println!("  Path:        {}", path);
        }
        None => {
            println!("Repository {} not found locally.", rid);
            println!("  Discovery via DHT is not yet implemented.");
            println!("  Start the node with 'fossil-p2p node start' to enable discovery.");
        }
    }
    Ok(())
}

fn cmd_repo_clone(rid: String, directory: PathBuf) -> Result<()> {
    let home = get_home()?;
    let db = get_db(&home)?;

    let (name, _desc, source_path, _owner, _vis, _fossil_db) = db
        .load_repository(&rid)?
        .ok_or_else(|| {
            crate::error::FossilP2pError::Repository(format!("repository not found: {}", rid))
        })?;

    let target_path = if directory.as_os_str() == "." {
        std::path::PathBuf::from(&name)
    } else {
        directory
    };

    let fossil = FossilCli::new(&FossilP2pConfig::default().fossil);
    let fossil_db = std::path::Path::new(&source_path)
        .join(format!("{}.fossil", name));

    if fossil_db.exists() {
        fossil.clone(
            &format!("file:{}", fossil_db.display()),
            &target_path,
        )?;
    } else {
        return Err(crate::error::FossilP2pError::Repository(format!(
            "fossil database not found at {}",
            fossil_db.display()
        )));
    }

    println!("Repository cloned!");
    println!("  RID:    {}", rid);
    println!("  Path:   {}", target_path.display());
    Ok(())
}

fn cmd_sync(rid: Option<String>) -> Result<()> {
    let home = get_home()?;
    let db = get_db(&home)?;

    let rid = rid.unwrap_or_else(|| {
        db.list_repositories()
            .unwrap_or_default()
            .first()
            .map(|(r, _, _, _)| r.clone())
            .unwrap_or_default()
    });
    if rid.is_empty() {
        println!("No repository specified or found.");
        return Ok(());
    }

    let (name, _desc, path, _owner, _vis, _fossil_db) =
        db.load_repository(&rid)?
            .ok_or_else(|| {
                crate::error::FossilP2pError::Repository(format!(
                    "repository not found: {}",
                    rid
                ))
            })?;

    println!("Syncing: {} ({})", name, rid);

    let fossil = FossilCli::new(&FossilP2pConfig::default().fossil);
    match fossil.sync(std::path::Path::new(&path), None) {
        Ok(output) => {
            if !output.trim().is_empty() {
                println!("{}", output);
            }
            println!("Sync complete.");
        }
        Err(e) => {
            println!("Sync status: {e}");
            println!("  Note: P2P sync is not yet connected to a remote.");
        }
    }
    Ok(())
}

fn cmd_config_show() -> Result<()> {
    let home = get_home()?;
    let config = get_config(&home)?;
    println!("{}", serde_json::to_string_pretty(&config)?);
    Ok(())
}

fn cmd_config_init() -> Result<()> {
    let home = get_home()?;
    home.init()?;
    let config = FossilP2pConfig::default();
    config.save(&home.config())?;
    println!("Configuration initialized at {}", home.config().display());
    Ok(())
}

fn cmd_config_get(key: String) -> Result<()> {
    let home = get_home()?;
    let config = get_config(&home)?;
    let value = match key.as_str() {
        "node.alias" => config.node.alias.clone(),
        "node.log" => config.node.log.clone(),
        "p2p.listen" => config.p2p.listen.join(", "),
        "p2p.kad_protocol" => config.p2p.kad_protocol.clone(),
        "p2p.relay_enabled" => config.p2p.relay_enabled.to_string(),
        "p2p.idle_timeout_secs" => config.p2p.idle_timeout_secs.to_string(),
        "fossil.fossil_path" => config.fossil.fossil_path.clone(),
        "fossil.http_port" => config.fossil.http_port.to_string(),
        _ => {
            return Err(crate::error::FossilP2pError::Config(format!(
                "unknown key: {key}"
            )));
        }
    };
    println!("{}", value);
    Ok(())
}

fn cmd_config_set(key: String, value: String) -> Result<()> {
    let home = get_home()?;
    let mut config = get_config(&home)?;
    match key.as_str() {
        "node.alias" => config.node.alias = value,
        "node.log" => config.node.log = value,
        "p2p.listen" => {
            config.p2p.listen = value.split(',').map(|s| s.trim().to_string()).collect()
        }
        "p2p.kad_protocol" => config.p2p.kad_protocol = value,
        "p2p.relay_enabled" => {
            config.p2p.relay_enabled = value
                .parse()
                .map_err(|e| crate::error::FossilP2pError::Config(format!("invalid bool: {e}")))?
        }
        "p2p.idle_timeout_secs" => {
            config.p2p.idle_timeout_secs = value.parse().map_err(|e| {
                crate::error::FossilP2pError::Config(format!("invalid u64: {e}"))
            })?
        }
        "fossil.fossil_path" => config.fossil.fossil_path = value,
        "fossil.http_port" => {
            config.fossil.http_port = value.parse().map_err(|e| {
                crate::error::FossilP2pError::Config(format!("invalid u16: {e}"))
            })?
        }
        _ => {
            return Err(crate::error::FossilP2pError::Config(format!(
                "unknown key: {key}"
            )));
        }
    }
    config.save(&home.config())?;
    println!("Configuration updated.");
    Ok(())
}

fn cmd_fossil_passthrough(args: Vec<String>) -> Result<()> {
    let fossil = FossilCli::new(&FossilP2pConfig::default().fossil);
    let args_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
    let output = std::process::Command::new(&fossil.fossil_path)
        .args(&args_refs)
        .output()
        .map_err(|e| {
            crate::error::FossilP2pError::Fossil(format!(
                "failed to execute fossil: {e}"
            ))
        })?;
    print!("{}", String::from_utf8_lossy(&output.stdout));
    eprint!("{}", String::from_utf8_lossy(&output.stderr));
    if !output.status.success() {
        std::process::exit(output.status.code().unwrap_or(1));
    }
    Ok(())
}

pub fn run_cli() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init => cmd_init(),
        Commands::Identity => cmd_identity(),
        Commands::Peer { command } => match command {
            PeerCommands::Add {
                public_key,
                alias,
                addresses,
            } => cmd_peer_add(public_key, alias, addresses),
            PeerCommands::List => cmd_peer_list(),
        },
        Commands::Node { command } => match command {
            NodeCommands::Start => cmd_node_start(),
            NodeCommands::Status => cmd_node_status(),
        },
        Commands::Repo { command } => match command {
            RepoCommands::List => cmd_repo_list(),
            RepoCommands::Publish {
                path,
                name,
                description,
            } => cmd_repo_publish(path, name, description),
            RepoCommands::Unpublish { rid: _ } => {
                println!("Unpublish is not yet implemented.");
                Ok(())
            }
            RepoCommands::Discover { rid } => cmd_repo_discover(rid),
            RepoCommands::Clone { rid, directory } => cmd_repo_clone(rid, directory),
        },
        Commands::Sync { rid } => cmd_sync(rid),
        Commands::Config { command } => match command {
            Some(ConfigCommands::Show) | None => cmd_config_show(),
            Some(ConfigCommands::Init) => cmd_config_init(),
            Some(ConfigCommands::Get { key }) => cmd_config_get(key),
            Some(ConfigCommands::Set { key, value }) => cmd_config_set(key, value),
        },
        Commands::Fossil { args } => cmd_fossil_passthrough(args),
        Commands::Transport {
            url,
            request_file,
            reply_file,
        } => {
            crate::transport::run_transport(&url, &request_file, &reply_file)?;
            Ok(())
        }
    }
}
