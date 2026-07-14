#![allow(dead_code, unused_imports, unused_variables)]

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, bail, Context, Result};
use clap::{Parser, Subcommand};
use ed25519_dalek::{Signer, SigningKey, Verifier, VerifyingKey};
use serde::{Deserialize, Serialize};

mod crypto {
    use super::*;

    #[derive(Clone)]
    pub struct Keypair {
        signing: SigningKey,
        verifying: [u8; 32],
    }

    impl Keypair {
        pub fn generate() -> Self {
            let mut csprng = rand_core::OsRng;
            let signing = SigningKey::generate(&mut csprng);
            let verifying = signing.verifying_key().to_bytes();
            Self { signing, verifying }
        }

        pub fn from_bytes(secret: &[u8; 32]) -> Result<Self> {
            let signing = SigningKey::from_bytes(secret);
            let verifying = signing.verifying_key().to_bytes();
            Ok(Self { signing, verifying })
        }

        pub fn public_key(&self) -> PublicKey {
            PublicKey(self.verifying)
        }

        pub fn secret_bytes(&self) -> [u8; 32] {
            self.signing.to_bytes()
        }

        pub fn sign(&self, msg: &[u8]) -> Signature {
            Signature(self.signing.sign(msg).to_bytes())
        }
    }

    #[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
    pub struct PublicKey([u8; 32]);

    impl PublicKey {
        pub fn to_bytes(&self) -> [u8; 32] {
            self.0
        }

        pub fn from_bytes(bytes: &[u8; 32]) -> Result<Self> {
            let _vk = VerifyingKey::from_bytes(bytes)
                .map_err(|e| anyhow!("invalid public key: {}", e))?;
            Ok(Self(*bytes))
        }

        pub fn verifying_key(&self) -> Result<VerifyingKey> {
            VerifyingKey::from_bytes(&self.0).map_err(|e| anyhow!("{}", e))
        }

        pub fn verify(&self, msg: &[u8], sig: &Signature) -> bool {
            if let Ok(vk) = self.verifying_key() {
                let ed_sig = ed25519_dalek::Signature::from_bytes(&sig.0);
                return vk.verify(msg, &ed_sig).is_ok();
            }
            false
        }

        pub fn to_multibase(&self) -> String {
            let mut buf = vec![0xED, 0x01];
            buf.extend_from_slice(&self.0);
            format!("z{}", bs58::encode(&buf).into_string())
        }

        pub fn from_multibase(s: &str) -> Result<Self> {
            let s = s.strip_prefix('z').ok_or_else(|| anyhow!("missing 'z' prefix"))?;
            let bytes = bs58::decode(s).into_vec()?;
            if bytes.len() < 34 || bytes[0] != 0xED || bytes[1] != 0x01 {
                bail!("invalid multicodec prefix");
            }
            let key_bytes: [u8; 32] = bytes[2..34]
                .try_into()
                .map_err(|_| anyhow!("invalid key length"))?;
            Self::from_bytes(&key_bytes)
        }

        pub fn to_hex(&self) -> String {
            hex::encode(self.0)
        }

        pub fn from_hex(s: &str) -> Result<Self> {
            let bytes = hex::decode(s)?;
            let arr: [u8; 32] = bytes
                .try_into()
                .map_err(|_| anyhow!("invalid key length"))?;
            Self::from_bytes(&arr)
        }
    }

    impl std::fmt::Display for PublicKey {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.to_multibase())
        }
    }

    impl std::fmt::Debug for PublicKey {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "PublicKey({})", self.to_multibase())
        }
    }

    #[derive(Clone, Copy)]
    pub struct Signature(pub [u8; 64]);

    impl Signature {
        pub fn to_bytes(&self) -> [u8; 64] {
            self.0
        }

        pub fn from_bytes(bytes: &[u8; 64]) -> Result<Self> {
            Ok(Self(*bytes))
        }

        pub fn to_hex(&self) -> String {
            hex::encode(self.0)
        }

        pub fn from_hex(s: &str) -> Result<Self> {
            let bytes = hex::decode(s)?;
            let arr: [u8; 64] = bytes
                .try_into()
                .map_err(|_| anyhow!("invalid signature length"))?;
            Self::from_bytes(&arr)
        }
    }

    impl std::fmt::Debug for Signature {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "Signature({})", self.to_hex())
        }
    }

    impl Serialize for Signature {
        fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
            serializer.serialize_str(&self.to_hex())
        }
    }

    impl<'de> Deserialize<'de> for Signature {
        fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
            let s = String::deserialize(deserializer)?;
            Signature::from_hex(&s).map_err(serde::de::Error::custom)
        }
    }
}

mod identity {
    use super::*;
    use crate::crypto::{Keypair, PublicKey, Signature};

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Did(pub String);

    impl Did {
        pub fn from_public_key(pk: &PublicKey) -> Self {
            Self(format!("did:key:{}", pk.to_multibase()))
        }
    }

    impl std::fmt::Display for Did {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.0)
        }
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Project {
        pub name: String,
        pub description: String,
        pub default_branch: String,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct IdentityDoc {
        pub version: u8,
        pub project: Project,
        pub delegates: Vec<String>,
        pub threshold: usize,
        pub visibility: Visibility,
        pub timestamp: String,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum Visibility {
        Public,
        Private { allow: Vec<String> },
    }

    impl IdentityDoc {
        pub fn new(
            project: Project,
            delegates: Vec<PublicKey>,
            threshold: usize,
            visibility: Visibility,
        ) -> Self {
            let now = time::OffsetDateTime::now_utc();
            let timestamp = now
                .format(&time::format_description::well_known::Rfc3339)
                .unwrap_or_default();
            Self {
                version: 1,
                project,
                delegates: delegates.iter().map(|pk| pk.to_multibase()).collect(),
                threshold,
                visibility,
                timestamp,
            }
        }

        pub fn to_json(&self) -> Result<String> {
            Ok(serde_json::to_string_pretty(self)?)
        }

        pub fn from_json(s: &str) -> Result<Self> {
            Ok(serde_json::from_str(s)?)
        }

        pub fn sign(&self, keypair: &Keypair) -> Result<Signature> {
            let json = self.to_json()?;
            Ok(keypair.sign(json.as_bytes()))
        }
    }
}

mod git {
    use super::*;
    use git2::{build::RepoBuilder, Repository as RawRepo};

    pub struct Repository {
        pub path: PathBuf,
        pub raw: RawRepo,
    }

    impl Repository {
        pub fn init(path: &Path) -> Result<Self> {
            let repo = RawRepo::init(path)?;
            Ok(Self { path: path.to_path_buf(), raw: repo })
        }

        pub fn init_bare(path: &Path) -> Result<Self> {
            let repo = RawRepo::init_bare(path)?;
            Ok(Self { path: path.to_path_buf(), raw: repo })
        }

        pub fn open(path: &Path) -> Result<Self> {
            let repo = RawRepo::open(path)?;
            Ok(Self { path: path.to_path_buf(), raw: repo })
        }

        pub fn head_oid(&self) -> Result<git2::Oid> {
            let head = self.raw.head().context("no HEAD reference")?;
            Ok(head.target().context("HEAD is not a direct reference")?)
        }

        pub fn head_branch_name(&self) -> Result<String> {
            let head = self.raw.head().context("no HEAD reference")?;
            let name = head.shorthand().context("HEAD is not a direct reference")?;
            Ok(name.to_string())
        }

        pub fn clone_remote(url: &str, path: &Path, branch: Option<&str>) -> Result<Self> {
            let mut opts = RepoBuilder::new();
            if let Some(b) = branch {
                opts.branch(b);
            }
            let repo = opts.clone(url, path)?;
            Ok(Self { path: path.to_path_buf(), raw: repo })
        }

        pub fn add_remote(&mut self, name: &str, url: &str) -> Result<()> {
            self.raw.remote(name, url).context("failed to add remote")?;
            Ok(())
        }

        pub fn set_push_url(&mut self, name: &str, url: &str) -> Result<()> {
            self.raw.remote_set_pushurl(name, Some(url))?;
            Ok(())
        }

        pub fn fetch(&mut self, remote_name: &str, _branch: Option<&str>) -> Result<()> {
            let mut remote = self.raw.find_remote(remote_name)?;
            let refspec = format!("refs/heads/*:refs/remotes/{}/*", remote_name);
            remote.fetch(&[&refspec], None, None)?;
            Ok(())
        }

        pub fn push(&mut self, remote_name: &str, branch: &str) -> Result<()> {
            let mut remote = self.raw.find_remote(remote_name)?;
            remote.push(&[&format!("refs/heads/{}", branch)], None)?;
            Ok(())
        }

        pub fn set_upstream(&mut self, branch: &str, remote: &str) -> Result<()> {
            let mut config = self.raw.config()?;
            config.set_str(&format!("branch.{}.remote", branch), remote)?;
            config.set_str(&format!("branch.{}.merge", branch), &format!("refs/heads/{}", branch))?;
            Ok(())
        }

        pub fn configure_push_default(&self) -> Result<()> {
            let mut config = self.raw.config()?;
            config.set_str("push.default", "upstream")?;
            Ok(())
        }

        pub fn has_remote(&self, name: &str) -> bool {
            self.raw.find_remote(name).is_ok()
        }

        pub fn remotes(&self) -> Vec<String> {
            let remotes = match self.raw.remotes() {
                Ok(r) => r,
                Err(_) => return Vec::new(),
            };
            let mut result = Vec::new();
            for i in 0..remotes.len() {
                if let Some(name) = remotes.get(i) {
                    result.push(name.to_string());
                }
            }
            result
        }

        pub fn add_all(&self) -> Result<git2::Oid> {
            let mut index = self.raw.index()?;
            index.add_all(["."], git2::IndexAddOption::DEFAULT, None)?;
            index.write()?;
            Ok(index.write_tree()?)
        }

        pub fn find_merge_base(&self, a: git2::Oid, b: git2::Oid) -> Result<git2::Oid> {
            Ok(self.raw.merge_base(a, b).context("no merge base found")?)
        }
    }
}

mod storage {
    use super::*;
    use rusqlite::{params, Connection};

    pub struct Database {
        pub conn: Connection,
    }

    impl Database {
        pub fn open(path: &Path) -> Result<Self> {
            let conn = Connection::open(path)?;
            let db = Self { conn };
            db.init_tables()?;
            Ok(db)
        }

        fn init_tables(&self) -> Result<()> {
            self.conn.execute_batch(
                "CREATE TABLE IF NOT EXISTS identity (
                    public_key TEXT PRIMARY KEY,
                    secret_key TEXT NOT NULL,
                    did TEXT NOT NULL,
                    alias TEXT,
                    created_at TEXT NOT NULL
                );
                CREATE TABLE IF NOT EXISTS repositories (
                    rid TEXT PRIMARY KEY,
                    name TEXT NOT NULL,
                    description TEXT,
                    default_branch TEXT NOT NULL,
                    path TEXT NOT NULL,
                    public_key TEXT NOT NULL,
                    created_at TEXT NOT NULL
                );
                CREATE TABLE IF NOT EXISTS known_peers (
                    public_key TEXT PRIMARY KEY,
                    alias TEXT,
                    addresses TEXT,
                    first_seen TEXT NOT NULL,
                    last_seen TEXT NOT NULL
                );
                CREATE TABLE IF NOT EXISTS refs (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    repo_rid TEXT NOT NULL,
                    peer_key TEXT NOT NULL,
                    ref_name TEXT NOT NULL,
                    oid TEXT NOT NULL
                );
                CREATE TABLE IF NOT EXISTS objects (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    repo_rid TEXT NOT NULL,
                    oid TEXT NOT NULL,
                    kind TEXT NOT NULL,
                    data BLOB
                );
                CREATE TABLE IF NOT EXISTS patches (
                    id TEXT PRIMARY KEY,
                    repo_rid TEXT NOT NULL,
                    title TEXT NOT NULL,
                    description TEXT,
                    author TEXT NOT NULL,
                    state TEXT NOT NULL DEFAULT 'open',
                    created_at TEXT NOT NULL
                );",
            )?;
            Ok(())
        }

        pub fn store_identity(&self, pk: &str, sk: &str, did: &str, alias: Option<&str>) -> Result<()> {
            let now = time::OffsetDateTime::now_utc()
                .format(&time::format_description::well_known::Rfc3339)
                .unwrap_or_default();
            self.conn.execute(
                "INSERT OR REPLACE INTO identity (public_key, secret_key, did, alias, created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
                params![pk, sk, did, alias, now],
            )?;
            Ok(())
        }

        pub fn load_identity(&self, pk: &str) -> Result<Option<(String, String, Option<String>)>> {
            let mut stmt = self.conn.prepare("SELECT secret_key, did, alias FROM identity WHERE public_key = ?1")?;
            let mut rows = stmt.query_map(params![pk], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?, row.get::<_, Option<String>>(2)?))
            })?;
            Ok(rows.next().transpose()?)
        }

        pub fn store_repository(&self, rid: &str, name: &str, description: Option<&str>, default_branch: &str, path: &str, public_key: &str) -> Result<()> {
            let now = time::OffsetDateTime::now_utc()
                .format(&time::format_description::well_known::Rfc3339)
                .unwrap_or_default();
            self.conn.execute(
                "INSERT OR REPLACE INTO repositories (rid, name, description, default_branch, path, public_key, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![rid, name, description, default_branch, path, public_key, now],
            )?;
            Ok(())
        }

        pub fn load_repository(&self, rid: &str) -> Result<Option<(String, Option<String>, String, String, String)>> {
            let mut stmt = self.conn.prepare("SELECT name, description, default_branch, path, public_key FROM repositories WHERE rid = ?1")?;
            let mut rows = stmt.query_map(params![rid], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, Option<String>>(1)?, row.get::<_, String>(2)?, row.get::<_, String>(3)?, row.get::<_, String>(4)?))
            })?;
            Ok(rows.next().transpose()?)
        }

        pub fn list_repositories(&self) -> Result<Vec<(String, String, Option<String>, String)>> {
            let mut stmt = self.conn.prepare("SELECT rid, name, description, default_branch FROM repositories ORDER BY name")?;
            let rows = stmt.query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?, row.get::<_, Option<String>>(2)?, row.get::<_, String>(3)?))
            })?;
            Ok(rows.filter_map(|r| r.ok()).collect())
        }

        pub fn store_peer(&self, pk: &str, alias: Option<&str>, addresses: Option<&str>) -> Result<()> {
            let now = time::OffsetDateTime::now_utc()
                .format(&time::format_description::well_known::Rfc3339)
                .unwrap_or_default();
            self.conn.execute(
                "INSERT OR REPLACE INTO known_peers (public_key, alias, addresses, first_seen, last_seen) VALUES (?1, ?2, ?3, ?4, ?4)",
                params![pk, alias, addresses, now],
            )?;
            Ok(())
        }

        pub fn list_peers(&self) -> Result<Vec<(String, Option<String>, Option<String>, String)>> {
            let mut stmt = self.conn.prepare("SELECT public_key, alias, addresses, last_seen FROM known_peers ORDER BY last_seen DESC")?;
            let rows = stmt.query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, Option<String>>(1)?, row.get::<_, Option<String>>(2)?, row.get::<_, String>(3)?))
            })?;
            Ok(rows.filter_map(|r| r.ok()).collect())
        }

        pub fn store_ref(&self, repo_rid: &str, peer_key: &str, ref_name: &str, oid: &str) -> Result<()> {
            self.conn.execute("DELETE FROM refs WHERE repo_rid = ?1 AND peer_key = ?2 AND ref_name = ?3", params![repo_rid, peer_key, ref_name])?;
            self.conn.execute("INSERT INTO refs (repo_rid, peer_key, ref_name, oid) VALUES (?1, ?2, ?3, ?4)", params![repo_rid, peer_key, ref_name, oid])?;
            Ok(())
        }

        pub fn load_refs(&self, repo_rid: &str, peer_key: &str) -> Result<Vec<(String, String)>> {
            let mut stmt = self.conn.prepare("SELECT ref_name, oid FROM refs WHERE repo_rid = ?1 AND peer_key = ?2")?;
            let rows = stmt.query_map(params![repo_rid, peer_key], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
            })?;
            Ok(rows.filter_map(|r| r.ok()).collect())
        }

        pub fn store_patch(&self, id: &str, repo_rid: &str, title: &str, description: Option<&str>, author: &str) -> Result<()> {
            let now = time::OffsetDateTime::now_utc()
                .format(&time::format_description::well_known::Rfc3339)
                .unwrap_or_default();
            self.conn.execute(
                "INSERT OR REPLACE INTO patches (id, repo_rid, title, description, author, state, created_at) VALUES (?1, ?2, ?3, ?4, ?5, 'open', ?6)",
                params![id, repo_rid, title, description, author, now],
            )?;
            Ok(())
        }

        pub fn load_patch(&self, id: &str) -> Result<Option<(String, Option<String>, String, String, String)>> {
            let mut stmt = self.conn.prepare("SELECT title, description, author, state, created_at FROM patches WHERE id = ?1")?;
            let mut rows = stmt.query_map(params![id], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, Option<String>>(1)?, row.get::<_, String>(2)?, row.get::<_, String>(3)?, row.get::<_, String>(4)?))
            })?;
            Ok(rows.next().transpose()?)
        }

        pub fn list_patches(&self, repo_rid: &str) -> Result<Vec<(String, String, String, String, String)>> {
            let mut stmt = self.conn.prepare("SELECT id, title, author, state, created_at FROM patches WHERE repo_rid = ?1 ORDER BY created_at DESC")?;
            let rows = stmt.query_map(params![repo_rid], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?, row.get::<_, String>(2)?, row.get::<_, String>(3)?, row.get::<_, String>(4)?))
            })?;
            Ok(rows.filter_map(|r| r.ok()).collect())
        }

        pub fn update_patch_state(&self, id: &str, state: &str) -> Result<()> {
            self.conn.execute("UPDATE patches SET state = ?1 WHERE id = ?2", params![state, id])?;
            Ok(())
        }
    }
}

mod protocol {
    use super::*;
    use crate::crypto::{PublicKey, Signature};

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum Message {
        Ping,
        Pong,
        Subscribe(Subscribe),
        Announcement(Announcement),
        Info(Info),
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Subscribe {
        pub filter: Vec<String>,
        pub since: String,
        pub until: String,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Announcement {
        pub node: PublicKey,
        pub signature: Signature,
        pub message: AnnouncementMessage,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum AnnouncementMessage {
        Inventory(InventoryAnnouncement),
        Refs(RefsAnnouncement),
        Node(NodeAnnouncement),
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct InventoryAnnouncement {
        pub inventory: Vec<String>,
        pub timestamp: String,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct RefsAnnouncement {
        pub rid: String,
        pub refs: Vec<RefsAt>,
        pub timestamp: String,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct RefsAt {
        pub remote: String,
        pub at: String,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct NodeAnnouncement {
        pub version: u8,
        pub timestamp: String,
        pub alias: String,
        pub addresses: Vec<String>,
        pub nonce: u64,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum Info {
        RefsAlreadySynced { rid: String, at: String },
    }

    pub fn serialize_message(msg: &Message) -> Result<Vec<u8>> {
        let mut buf = Vec::new();
        ciborium::into_writer(msg, &mut buf)?;
        Ok(buf)
    }

    pub fn deserialize_message(data: &[u8]) -> Result<Message> {
        Ok(ciborium::from_reader(data)?)
    }
}

mod peer {
    use super::*;
    use crate::crypto::{Keypair, PublicKey};
    use crate::protocol::{Announcement, AnnouncementMessage, NodeAnnouncement};

    pub struct PeerManager {
        pub keypair: Keypair,
        pub alias: String,
        pub addresses: Vec<String>,
        pub peers: Vec<KnownPeer>,
    }

    #[derive(Debug, Clone)]
    pub struct KnownPeer {
        pub public_key: PublicKey,
        pub alias: Option<String>,
        pub addresses: Vec<String>,
        pub first_seen: String,
        pub last_seen: String,
    }

    impl PeerManager {
        pub fn new(keypair: Keypair, alias: String, addresses: Vec<String>) -> Self {
            Self { keypair, alias, addresses, peers: Vec::new() }
        }

        pub fn add_peer(&mut self, public_key: PublicKey, alias: Option<String>, addresses: Vec<String>) -> Result<()> {
            let now = time::OffsetDateTime::now_utc()
                .format(&time::format_description::well_known::Rfc3339)
                .unwrap_or_default();
            if let Some(peer) = self.peers.iter_mut().find(|p| p.public_key == public_key) {
                peer.last_seen = now;
                if let Some(a) = alias { peer.alias = Some(a); }
                if !addresses.is_empty() { peer.addresses = addresses; }
            } else {
                self.peers.push(KnownPeer {
                    public_key,
                    alias,
                    addresses,
                    first_seen: now.clone(),
                    last_seen: now,
                });
            }
            Ok(())
        }

        pub fn remove_peer(&mut self, public_key: &PublicKey) -> bool {
            let before = self.peers.len();
            self.peers.retain(|p| p.public_key != *public_key);
            self.peers.len() < before
        }

        pub fn list_peers(&self) -> &[KnownPeer] {
            &self.peers
        }

        pub fn create_node_announcement(&self) -> Result<NodeAnnouncement> {
            use rand_core::RngCore;
            let now = time::OffsetDateTime::now_utc()
                .format(&time::format_description::well_known::Rfc3339)
                .unwrap_or_default();
            let mut rng = rand_core::OsRng;
            Ok(NodeAnnouncement {
                version: 1,
                timestamp: now,
                alias: self.alias.clone(),
                addresses: self.addresses.clone(),
                nonce: rng.next_u64(),
            })
        }

        pub fn sign_announcement(&self, msg: &AnnouncementMessage) -> Result<Announcement> {
            let json = serde_json::to_vec(msg)?;
            let sig = self.keypair.sign(&json);
            Ok(Announcement {
                node: self.keypair.public_key(),
                signature: sig,
                message: msg.clone(),
            })
        }

        pub fn verify_announcement(&self, announcement: &Announcement) -> bool {
            let json = serde_json::to_vec(&announcement.message).unwrap_or_default();
            announcement.node.verify(&json, &announcement.signature)
        }
    }
}

mod config {
    use super::*;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct RadConfig {
        #[serde(default)]
        pub public_explorer: String,
        #[serde(default)]
        pub preferred_seeds: Vec<String>,
        #[serde(default)]
        pub node: NodeConfig,
        #[serde(default)]
        pub cli: CliConfig,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct NodeConfig {
        #[serde(default = "default_alias")]
        pub alias: String,
        #[serde(default)]
        pub listen: Vec<String>,
        #[serde(default = "default_peers_type")]
        pub peers_type: String,
        #[serde(default)]
        pub connect: Vec<String>,
        #[serde(default)]
        pub external_addresses: Vec<String>,
        #[serde(default = "default_network")]
        pub network: String,
        #[serde(default = "default_log_level")]
        pub log: String,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct CliConfig {
        #[serde(default = "default_true")]
        pub hints: bool,
    }

    fn default_alias() -> String { "radicle-peer".to_string() }
    fn default_peers_type() -> String { "dynamic".to_string() }
    fn default_network() -> String { "main".to_string() }
    fn default_log_level() -> String { "INFO".to_string() }
    fn default_true() -> bool { true }

    impl Default for NodeConfig {
        fn default() -> Self {
            Self {
                alias: default_alias(),
                listen: vec!["127.0.0.1:8776".to_string()],
                peers_type: default_peers_type(),
                connect: vec![],
                external_addresses: vec![],
                network: default_network(),
                log: default_log_level(),
            }
        }
    }

    impl Default for CliConfig {
        fn default() -> Self { Self { hints: true } }
    }

    impl Default for RadConfig {
        fn default() -> Self {
            Self {
                public_explorer: "https://app.radicle.example.com/nodes/$host/$rid$path".to_string(),
                preferred_seeds: vec![
                    "z6MkrLMMsiPWUcNPHcRajuMi9mDfYckSoJyPwwnknocNYPm7@iris.radicle.network:8776".to_string(),
                    "z6Mkmqogy2qEM2ummccUthFEaaHvyYmYBYh3dbe9W4ebScxo@rosa.radicle.network:8776".to_string(),
                ],
                node: NodeConfig::default(),
                cli: CliConfig::default(),
            }
        }
    }

    impl RadConfig {
        pub fn load(path: &Path) -> Result<Self> {
            if path.exists() {
                let content = fs::read_to_string(path)?;
                Ok(serde_json::from_str(&content)?)
            } else {
                Ok(Self::default())
            }
        }

        pub fn save(&self, path: &Path) -> Result<()> {
            let content = serde_json::to_string_pretty(self)?;
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(path, content)?;
            Ok(())
        }
    }
}

mod home {
    use super::*;

    #[derive(Debug, Clone)]
    pub struct Home {
        pub path: PathBuf,
    }

    impl Home {
        pub fn new() -> Result<Self> {
            let path = if let Ok(rad_home) = std::env::var("RAD_HOME") {
                PathBuf::from(rad_home)
            } else {
                let dirs = directories::ProjectDirs::from("", "", "radicle")
                    .ok_or_else(|| anyhow!("cannot determine home directory"))?;
                dirs.data_dir().to_path_buf()
            };
            Ok(Self { path })
        }

        pub fn storage(&self) -> PathBuf { self.path.join("storage") }
        pub fn config(&self) -> PathBuf { self.path.join("config.json") }
        pub fn keys(&self) -> PathBuf { self.path.join("keys") }
        pub fn db(&self) -> PathBuf { self.path.join("node.db") }

        pub fn init(&self) -> Result<()> {
            fs::create_dir_all(self.storage())?;
            fs::create_dir_all(self.keys())?;
            fs::create_dir_all(self.path.join("node"))?;
            fs::create_dir_all(self.path.join("cobs"))?;
            Ok(())
        }

        pub fn secret_key_path(&self) -> PathBuf { self.keys().join("radicle") }
        pub fn public_key_path(&self) -> PathBuf { self.keys().join("radicle.pub") }
    }
}

use crate::config::RadConfig;
use crate::crypto::{Keypair, PublicKey};
use crate::home::Home;
use crate::identity::Did;
use crate::storage::Database;

#[derive(Parser)]
#[command(name = "rad")]
#[command(version = "0.1.0")]
#[command(about = "A minimal Radicle-inspired distributed code collaboration tool")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Init {
        #[arg(default_value = ".")]
        path: PathBuf,
        #[arg(short, long)]
        name: Option<String>,
        #[arg(short, long)]
        description: Option<String>,
        #[arg(short, long, default_value = "main")]
        branch: String,
    },
    Clone {
        rid: String,
        #[arg(default_value = ".")]
        directory: PathBuf,
    },
    Push {
        #[arg(default_value = "rad")]
        remote: String,
        branch: Option<String>,
    },
    Fetch {
        #[arg(default_value = "rad")]
        remote: String,
        branch: Option<String>,
    },
    Peer {
        #[command(subcommand)]
        command: PeerCommands,
    },
    Id,
    Patch {
        #[command(subcommand)]
        command: PatchCommands,
    },
    Config {
        #[command(subcommand)]
        command: Option<ConfigCommands>,
    },
    Status,
    Sync {
        rid: Option<String>,
        #[arg(short, long)]
        fetch: bool,
        #[arg(short, long)]
        announce: bool,
    },
}

#[derive(Subcommand)]
enum PeerCommands {
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
enum PatchCommands {
    Create {
        #[arg(short, long)]
        title: String,
        #[arg(short, long)]
        description: Option<String>,
        #[arg(short, long)]
        repo: Option<String>,
    },
    List {
        #[arg(short, long)]
        repo: Option<String>,
    },
    Merge {
        id: String,
    },
}

#[derive(Subcommand)]
enum ConfigCommands {
    Show,
    Init,
    Get { key: String },
    Set { key: String, value: String },
}

fn get_home() -> Result<Home> { Home::new() }
fn get_db(home: &Home) -> Result<Database> { Database::open(&home.db()) }

fn get_keypair(home: &Home) -> Result<Keypair> {
    let sk_path = home.secret_key_path();
    if sk_path.exists() {
        let sk_hex = fs::read_to_string(&sk_path)?;
        let sk_bytes = hex::decode(sk_hex.trim())?;
        let sk_arr: [u8; 32] = sk_bytes.try_into().map_err(|_| anyhow!("invalid secret key"))?;
        Keypair::from_bytes(&sk_arr)
    } else {
        let keypair = Keypair::generate();
        fs::create_dir_all(home.keys())?;
        fs::write(&sk_path, hex::encode(keypair.secret_bytes()))?;
        fs::write(home.public_key_path(), hex::encode(keypair.public_key().to_bytes()))?;
        Ok(keypair)
    }
}

fn get_config(home: &Home) -> Result<RadConfig> { RadConfig::load(&home.config()) }

fn cmd_init(path: PathBuf, name: Option<String>, description: Option<String>, branch: String) -> Result<()> {
    let home = get_home()?;
    home.init()?;
    let keypair = get_keypair(&home)?;
    let db = get_db(&home)?;

    let repo_path = if path == PathBuf::from(".") { std::env::current_dir()? } else { path };

    if !repo_path.join(".git").exists() {
        let repo = git2::Repository::init(&repo_path)?;
        let mut cfg = repo.config()?;
        cfg.set_str("user.name", &keypair.public_key().to_string())?;
        cfg.set_str("user.email", &format!("{}@radicle.local", keypair.public_key()))?;
    }

    let project_name = name
        .or_else(|| repo_path.file_name().and_then(|n| n.to_str()).map(|s| s.to_string()))
        .unwrap_or_else(|| "my-project".to_string());

    let working_repo = git::Repository::open(&repo_path)?;
    let head_oid = working_repo.head_oid()?;
    let pk = keypair.public_key();

    let project = identity::Project {
        name: project_name.clone(),
        description: description.unwrap_or_default(),
        default_branch: branch.clone(),
    };

    let doc = identity::IdentityDoc::new(project, vec![pk], 1, identity::Visibility::Public);
    let doc_json = doc.to_json()?;

    let mut hasher = sha2::Sha256::new();
    use sha2::Digest;
    hasher.update(doc_json.as_bytes());
    let hash = hasher.finalize();
    let rid_bytes: [u8; 32] = hash.into();
    let rid_hex = hex::encode(rid_bytes);

    let storage_path = home.storage().join(&rid_hex);
    let storage_repo = git::Repository::init_bare(&storage_path)?;

    let namespace = pk.to_multibase();

    {
        let mut config = storage_repo.raw.config()?;
        config.set_str("user.name", &project_name)?;
        config.set_str("user.email", &format!("{}@radicle.local", namespace))?;
        config.set_str("push.default", "upstream")?;
    }
    drop(storage_repo);

    std::process::Command::new("git")
        .arg("push")
        .arg(format!("{}:refs/heads/{}", repo_path.display(), branch))
        .arg(storage_path.to_str().unwrap_or_default())
        .current_dir(&repo_path)
        .output()
        .ok();

    let storage_repo = git::Repository::open(&storage_path)?;
    let namespace_ref = format!("refs/namespaces/{}/refs/heads/{}", namespace, branch);
    if storage_repo.raw.refname_to_id(&namespace_ref).is_err() {
        if let Ok(ref_name) = storage_repo.raw.refname_to_id(&format!("refs/heads/{}", branch)) {
            storage_repo.raw.reference(&namespace_ref, ref_name, true, "init namespace")?;
        } else if let Ok(oid) = storage_repo.raw.refname_to_id("HEAD") {
            storage_repo.raw.reference(&namespace_ref, oid, true, "init namespace")?;
        }
    }

    let mut working_repo = git::Repository::open(&repo_path)?;
    let rad_url = format!("rad://{}", rid_hex);
    let rad_push_url = format!("rad://{}/{}", rid_hex, namespace);
    working_repo.add_remote("rad", &rad_url)?;
    working_repo.set_push_url("rad", &rad_push_url)?;
    working_repo.configure_push_default()?;

    db.store_identity(&pk.to_multibase(), &hex::encode(keypair.secret_bytes()), &Did::from_public_key(&pk).to_string(), Some(&project_name))?;
    db.store_repository(&rid_hex, &project_name, None, &branch, &repo_path.to_string_lossy(), &pk.to_multibase())?;

    println!("Repository initialized successfully!");
    println!("  RID:      {}", rid_hex);
    println!("  Identity: {}", pk);
    println!("  DID:      {}", Did::from_public_key(&pk));
    println!("  Branch:   {}", branch);
    println!("  Storage:  {}", storage_path.display());
    Ok(())
}

fn cmd_clone(rid: String, directory: PathBuf) -> Result<()> {
    let home = get_home()?;
    home.init()?;
    let keypair = get_keypair(&home)?;
    let db = get_db(&home)?;

    let (name, _desc, default_branch, source_path, _pk) = db
        .load_repository(&rid)?
        .ok_or_else(|| anyhow!("repository not found: {}", rid))?;

    let target_path = if directory == PathBuf::from(".") { PathBuf::from(&name) } else { directory };

    let source_repo = git::Repository::open(&Path::new(&source_path))?;
    git::Repository::init(&target_path)?;

    let pk = keypair.public_key();
    let namespace = pk.to_multibase();
    let branch_ref = format!("refs/namespaces/{}/refs/heads/{}", namespace, default_branch);
    let head_oid = source_repo.raw.refname_to_id(&branch_ref)?;

    let mut target_repo = git::Repository::open(&target_path)?;
    target_repo.raw.reference(&format!("refs/heads/{}", default_branch), head_oid, true, "clone")?;
    target_repo.raw.reference_symbolic("HEAD", &format!("refs/heads/{}", default_branch), false, "clone")?;

    let rad_url = format!("rad://{}", rid);
    let rad_push_url = format!("rad://{}/{}", rid, namespace);
    target_repo.add_remote("rad", &rad_url)?;
    target_repo.set_push_url("rad", &rad_push_url)?;
    target_repo.configure_push_default()?;

    let tree = target_repo.raw.find_tree(target_repo.raw.find_commit(head_oid)?.tree_id())?;
    target_repo.raw.checkout_tree(tree.as_object(), None)?;

    println!("Repository cloned successfully!");
    println!("  RID:     {}", rid);
    println!("  Path:    {}", target_path.display());
    println!("  Branch:  {}", default_branch);
    Ok(())
}

fn cmd_push(remote: String, branch: Option<String>) -> Result<()> {
    let repo = git::Repository::open(&std::env::current_dir()?)?;
    let branch_name = branch.unwrap_or_else(|| repo.head_branch_name().unwrap_or_else(|_| "main".to_string()));
    let mut repo = git::Repository::open(&std::env::current_dir()?)?;
    repo.push(&remote, &branch_name)?;
    println!("Pushed to {} / {}", remote, branch_name);
    Ok(())
}

fn cmd_fetch(remote: String, branch: Option<String>) -> Result<()> {
    let mut repo = git::Repository::open(&std::env::current_dir()?)?;
    repo.fetch(&remote, branch.as_deref())?;
    println!("Fetched from {}", remote);
    Ok(())
}

fn cmd_peer_add(public_key: String, alias: Option<String>, addresses: Vec<String>) -> Result<()> {
    let home = get_home()?;
    let db = get_db(&home)?;
    let pk = PublicKey::from_multibase(&public_key)?;
    let address_str = if addresses.is_empty() { None } else { Some(addresses.join(",")) };
    db.store_peer(&pk.to_multibase(), alias.as_deref(), address_str.as_deref())?;
    println!("Peer added: {}", pk);
    if let Some(a) = &alias { println!("  Alias:    {}", a); }
    if !addresses.is_empty() { println!("  Addresses: {}", addresses.join(", ")); }
    Ok(())
}

fn cmd_peer_list() -> Result<()> {
    let home = get_home()?;
    let db = get_db(&home)?;
    let peers = db.list_peers()?;
    if peers.is_empty() { println!("No known peers."); return Ok(()); }
    println!("{:<70} {:<20} {:<30}", "PUBLIC KEY", "ALIAS", "LAST SEEN");
    println!("{}", "-".repeat(122));
    for (pk, alias, _addresses, last_seen) in &peers {
        let alias_str = alias.as_deref().unwrap_or("-");
        let last_seen_short = if last_seen.len() > 10 { &last_seen[..10] } else { last_seen };
        println!("{:<70} {:<20} {:<30}", pk, alias_str, last_seen_short);
    }
    Ok(())
}

fn cmd_id() -> Result<()> {
    let home = get_home()?;
    let keypair = get_keypair(&home)?;
    let pk = keypair.public_key();
    let did = Did::from_public_key(&pk);
    println!("Identity Information:");
    println!("  Public Key: {}", pk);
    println!("  DID:        {}", did);
    println!("  Key Path:   {}", home.secret_key_path().display());
    Ok(())
}

fn cmd_patch_create(title: String, description: Option<String>, repo: Option<String>) -> Result<()> {
    let home = get_home()?;
    let db = get_db(&home)?;
    let keypair = get_keypair(&home)?;

    let rid = repo.unwrap_or_else(|| {
        db.list_repositories().unwrap_or_default()
            .first().map(|(r, _, _, _)| r.clone()).unwrap_or_default()
    });
    if rid.is_empty() { bail!("no repository found. Run 'rad init' first."); }

    let pk = keypair.public_key();
    let patch_id = uuid::Uuid::new_v4().to_string();
    db.store_patch(&patch_id, &rid, &title, description.as_deref(), &pk.to_multibase())?;

    println!("Patch created!");
    println!("  ID:    {}", patch_id);
    println!("  Title: {}", title);
    println!("  Repo:  {}", rid);
    Ok(())
}

fn cmd_patch_list(repo: Option<String>) -> Result<()> {
    let home = get_home()?;
    let db = get_db(&home)?;

    let rid = repo.unwrap_or_else(|| {
        db.list_repositories().unwrap_or_default()
            .first().map(|(r, _, _, _)| r.clone()).unwrap_or_default()
    });
    if rid.is_empty() { println!("No repository specified."); return Ok(()); }

    let patches = db.list_patches(&rid)?;
    if patches.is_empty() { println!("No patches found for repository {}.", rid); return Ok(()); }

    println!("{:<38} {:<40} {:<20} {:<10}", "ID", "TITLE", "AUTHOR", "STATE");
    println!("{}", "-".repeat(110));
    for (id, title, author, state, _created) in &patches {
        let id_short = if id.len() > 8 { &id[..8] } else { id };
        let title_short = if title.len() > 37 { format!("{}...", &title[..34]) } else { title.clone() };
        let author_short = if author.len() > 19 { format!("{}...", &author[..16]) } else { author.clone() };
        println!("{:<38} {:<40} {:<20} {:<10}", id_short, title_short, author_short, state);
    }
    Ok(())
}

fn cmd_patch_merge(id: String) -> Result<()> {
    let home = get_home()?;
    let db = get_db(&home)?;
    let (title, _desc, _author, state, _created) = db.load_patch(&id)?.ok_or_else(|| anyhow!("patch not found: {}", id))?;
    if state == "merged" { bail!("patch is already merged"); }
    db.update_patch_state(&id, "merged")?;
    println!("Patch merged!");
    println!("  ID:    {}", id);
    println!("  Title: {}", title);
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
    let config = RadConfig::default();
    config.save(&home.config())?;
    println!("Configuration initialized at {}", home.config().display());
    Ok(())
}

fn cmd_config_get(key: String) -> Result<()> {
    let home = get_home()?;
    let config = get_config(&home)?;
    let value = match key.as_str() {
        "publicExplorer" => config.public_explorer.clone(),
        "node.alias" => config.node.alias.clone(),
        "node.network" => config.node.network.clone(),
        "node.log" => config.node.log.clone(),
        "cli.hints" => config.cli.hints.to_string(),
        _ => bail!("unknown key: {}", key),
    };
    println!("{}", value);
    Ok(())
}

fn cmd_config_set(key: String, value: String) -> Result<()> {
    let home = get_home()?;
    let mut config = get_config(&home)?;
    match key.as_str() {
        "publicExplorer" => config.public_explorer = value,
        "node.alias" => config.node.alias = value,
        "node.network" => config.node.network = value,
        "node.log" => config.node.log = value,
        "cli.hints" => config.cli.hints = value.parse()?,
        _ => bail!("unknown key: {}", key),
    }
    config.save(&home.config())?;
    println!("Configuration updated.");
    Ok(())
}

fn cmd_status() -> Result<()> {
    let repo = git::Repository::open(&std::env::current_dir()?)?;
    let head_oid = repo.head_oid()?;
    let head_branch = repo.head_branch_name()?;
    println!("Repository Status:");
    println!("  Branch:  {}", head_branch);
    println!("  HEAD:    {}", head_oid);
    let remotes = repo.remotes();
    if remotes.is_empty() {
        println!("  Remotes: none");
    } else {
        println!("  Remotes:");
        for remote in &remotes { println!("    - {}", remote); }
    }
    if let Ok(url) = repo.raw.config().and_then(|c| c.get_string("remote.rad.url")) {
        println!("  Rad URL: {}", url);
    }
    Ok(())
}

fn cmd_sync(rid: Option<String>, fetch_only: bool, announce_only: bool) -> Result<()> {
    let home = get_home()?;
    let db = get_db(&home)?;

    let rid = rid.unwrap_or_else(|| {
        db.list_repositories().unwrap_or_default()
            .first().map(|(r, _, _, _)| r.clone()).unwrap_or_default()
    });
    if rid.is_empty() { bail!("no repository specified or found."); }

    let (name, _desc, _branch, path, _pk) = db.load_repository(&rid)?
        .ok_or_else(|| anyhow!("repository not found: {}", rid))?;

    println!("Syncing repository: {} ({})", name, rid);

    if !fetch_only { println!("  Announcing inventory..."); }
    if !announce_only {
        println!("  Fetching updates...");
        let mut repo = git::Repository::open(&Path::new(&path))?;
        if repo.has_remote("origin") {
            repo.fetch("origin", None)?;
        }
    }
    println!("Sync complete.");
    Ok(())
}

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "warn".into()))
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Init { path, name, description, branch } => cmd_init(path, name, description, branch),
        Commands::Clone { rid, directory } => cmd_clone(rid, directory),
        Commands::Push { remote, branch } => cmd_push(remote, branch),
        Commands::Fetch { remote, branch } => cmd_fetch(remote, branch),
        Commands::Peer { command } => match command {
            PeerCommands::Add { public_key, alias, addresses } => cmd_peer_add(public_key, alias, addresses),
            PeerCommands::List => cmd_peer_list(),
        },
        Commands::Id => cmd_id(),
        Commands::Patch { command } => match command {
            PatchCommands::Create { title, description, repo } => cmd_patch_create(title, description, repo),
            PatchCommands::List { repo } => cmd_patch_list(repo),
            PatchCommands::Merge { id } => cmd_patch_merge(id),
        },
        Commands::Config { command } => match command {
            Some(ConfigCommands::Show) | None => cmd_config_show(),
            Some(ConfigCommands::Init) => cmd_config_init(),
            Some(ConfigCommands::Get { key }) => cmd_config_get(key),
            Some(ConfigCommands::Set { key, value }) => cmd_config_set(key, value),
        },
        Commands::Status => cmd_status(),
        Commands::Sync { rid, fetch, announce } => cmd_sync(rid, fetch, announce),
    }
}
