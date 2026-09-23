pub mod commands;

pub use commands::run_cli;

use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "peergit")]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(about = "P2P transport, discovery, identity and collaboration layer for Fossil repositories")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Create the PeerGit home directory and node identity.
    Init,
    /// Show the node identity (DID, public key, PeerId).
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
    /// Pass arguments through to the Fossil binary.
    Fossil {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    /// Fossil transport adapter (used internally by `fossil --transport-command`).
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
    Remove {
        peer_id: String,
    },
}

#[derive(Subcommand)]
pub enum NodeCommands {
    Start,
    Status,
}

#[derive(Subcommand)]
pub enum RepoCommands {
    List,
    /// Create a new Fossil repository with a PeerGit identity.
    Init {
        #[arg(default_value = ".")]
        path: PathBuf,
        #[arg(short, long)]
        name: Option<String>,
        #[arg(short, long)]
        description: Option<String>,
        #[arg(short, long, value_parser = ["public", "private", "protected"])]
        visibility: Option<String>,
    },
    /// Register an existing Fossil repository with PeerGit.
    Publish {
        #[arg(default_value = ".")]
        path: PathBuf,
        #[arg(short, long)]
        name: Option<String>,
        #[arg(short, long)]
        description: Option<String>,
        #[arg(short, long, value_parser = ["public", "private", "protected"])]
        visibility: Option<String>,
    },
    Unpublish {
        rid: String,
    },
    /// Discover a repository via the DHT.
    Discover {
        rid: String,
    },
    /// Clone a repository from a peer discovered via the DHT.
    Clone {
        rid: String,
        #[arg(default_value = ".")]
        directory: PathBuf,
        /// Skip DHT lookup and connect directly to this peer (peer-id, alias, or multiaddr).
        #[arg(long)]
        peer: Option<String>,
    },
    /// Change repository visibility.
    Visibility {
        rid: String,
        #[arg(value_parser = ["public", "private", "protected"])]
        visibility: String,
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

pub fn visibility_from_str(s: &str) -> crate::identity::Visibility {
    match s {
        "private" => crate::identity::Visibility::Private { allow: vec![] },
        "protected" => crate::identity::Visibility::Protected,
        _ => crate::identity::Visibility::Public,
    }
}