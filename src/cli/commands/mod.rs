pub mod config;
pub mod identity;
pub mod node;
pub mod peer;
pub mod repo;
pub mod sync;
pub mod util;

use crate::config::FossilP2pConfig;
use crate::error::Result;
use crate::fossil::FossilCli;
use clap::Parser;

use crate::cli::{Cli, Commands, ConfigCommands, NodeCommands, PeerCommands, RepoCommands};

pub fn run_cli() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init => identity::cmd_init(),
        Commands::Identity => identity::cmd_identity(),
        Commands::Peer { command } => match command {
            PeerCommands::Add {
                public_key,
                alias,
                addresses,
            } => peer::cmd_peer_add(public_key, alias, addresses),
            PeerCommands::List => peer::cmd_peer_list(),
            PeerCommands::Remove { peer_id } => peer::cmd_peer_remove(peer_id),
        },
        Commands::Node { command } => match command {
            NodeCommands::Start => node::cmd_node_start(),
            NodeCommands::Status => node::cmd_node_status(),
        },
        Commands::Repo { command } => match command {
            RepoCommands::List => repo::cmd_repo_list(),
            RepoCommands::Init {
                path,
                name,
                description,
                visibility,
            } => repo::cmd_repo_init(path, name, description, visibility),
            RepoCommands::Publish {
                path,
                name,
                description,
                visibility,
            } => repo::cmd_repo_publish(path, name, description, visibility),
            RepoCommands::Unpublish { rid } => repo::cmd_repo_unpublish(rid),
            RepoCommands::Discover { rid } => repo::cmd_repo_discover(rid),
            RepoCommands::Clone {
                rid,
                directory,
                peer,
            } => repo::cmd_repo_clone(rid, directory, peer),
            RepoCommands::Visibility { rid, visibility } => {
                repo::cmd_repo_visibility(rid, visibility)
            }
        },
        Commands::Sync { rid } => sync::cmd_sync(rid),
        Commands::Config { command } => match command {
            Some(ConfigCommands::Show) | None => config::cmd_config_show(),
            Some(ConfigCommands::Init) => config::cmd_config_init(),
            Some(ConfigCommands::Get { key }) => config::cmd_config_get(key),
            Some(ConfigCommands::Set { key, value }) => config::cmd_config_set(key, value),
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

fn cmd_fossil_passthrough(args: Vec<String>) -> Result<()> {
    let fossil = FossilCli::new(&FossilP2pConfig::default().fossil);
    let args_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
    let output = std::process::Command::new(&fossil.fossil_path)
        .args(&args_refs)
        .output()
        .map_err(|e| crate::error::FossilP2pError::Fossil(format!("failed to execute fossil: {e}")))?;
    print!("{}", String::from_utf8_lossy(&output.stdout));
    eprint!("{}", String::from_utf8_lossy(&output.stderr));
    if !output.status.success() {
        std::process::exit(output.status.code().unwrap_or(1));
    }
    Ok(())
}