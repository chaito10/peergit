# Architecture

This document describes the internal architecture of Rad and how it compares to the full Heartwood implementation.

---

## Design Philosophy

Rad flattens the multi-crate Heartwood architecture into a single executable with logical modules. This makes the codebase easier to understand and experiment with while preserving the essential functionality.

!!! note "Reference Implementation"
    Rad is not a production-ready implementation. It is designed for learning, experimentation, and as a starting point for building on the Radicle protocol.

---

## Module Overview

```
src/main.rs
  mod crypto      -- Ed25519 keys, signing, verification
  mod identity    -- DID, project metadata, identity documents
  mod git         -- Git operations via libgit2
  mod storage     -- SQLite database
  mod protocol    -- CBOR network messages
  mod peer        -- Peer management
  mod config      -- JSON configuration
  mod home        -- Directory management
```

---

## Module Details

### crypto

Handles cryptographic operations:

- **Key generation**: Ed25519 keypairs via `ed25519-dalek`
- **Signing**: Message signing for announcements and patches
- **Verification**: Signature verification for received messages
- **Encoding**: Multibase encoding for public keys (Radicle-compatible)

```rust
// Example: Generating a new keypair
let mut csprng = OsRng;
let keypair = Keypair::generate(&mut csprng);

// Encoding public key
let public_key = multibase::encode(Base::Base32Z, &keypair.public.to_bytes());
```

### identity

Manages decentralized identities:

- **DID**: `did:key` identifiers derived from public keys
- **Project metadata**: Repository name, description, revision, visibility
- **Identity documents**: JSON-serialized identity with cryptographic proofs
- **DID documents**: W3C-compliant DID documents for verification

```rust
// Identity document structure
pub struct IdentityDocument {
    pub id: Did,
    #[serde(rename = "alsoKnownAs")]
    pub also_known_as: Vec<Did>,
    pub keys: Vec<DocumentKey>,
    #[serde(rename = "service")]
    pub services: Vec<Service>,
}
```

### git

Wraps `git2` (libgit2) for Git operations:

- **Repository init**: Create new repositories
- **Cloning**: Clone from storage or remotes
- **Fetching**: Pull changes from remotes
- **Pushing**: Push changes to remotes
- **References**: Manage branches, tags, and heads
- **Remotes**: Configure and manage remote connections

### storage

SQLite database for persistent state:

- **Identities**: DID documents and public keys
- **Repositories**: RID, name, description, visibility
- **Peers**: Node IDs, aliases, connection status
- **Refs**: Branch heads and commit hashes
- **Patches**: Title, description, author, state, commits

### protocol

CBOR-serialized network messages:

- **Ping**: Heartbeat messages
- **Pong**: Ping responses
- **Announcement**: Repository announcements with signatures
- **Inventory**: Repository and ref announcements

```rust
// Protocol message types
pub enum Message {
    Ping { value: u16 },
    Pong { value: u16 },
    Announcement {
        rid: RepositoryId,
        refs: BTreeMap<String, Oid>,
        timestamp: u64,
        signature: Signature,
        public_key: PublicKey,
    },
    Inventory {
        repositories: Vec<InventoryItem>,
        timestamp: u64,
        signature: Signature,
        public_key: PublicKey,
    },
}
```

### peer

Manages peer connections and repository exchange:

- **Announcement signing**: Sign repository state for broadcast
- **Announcement verification**: Verify received announcements
- **Inventory exchange**: Share repository lists with peers
- **Ref exchange**: Share branch heads with peers

### config

JSON configuration with Radicle-compatible defaults:

- **Public explorer**: URL template for web browsing
- **Preferred seeds**: Default seed node addresses
- **Node configuration**: Alias, listen address, network, log level
- **CLI configuration**: Hints and display options

### home

XDG-style home directory management:

- **Directory creation**: Config, keys, storage, node directories
- **File paths**: Configuration, keys, database paths
- **RAD_HOME override**: Environment variable support

---

## Comparison with Heartwood

| Aspect | Heartwood | Rad |
|--------|-----------|-----|
| **Structure** | Multi-crate workspace | Single file |
| **Identity** | SHA-1 OID (Git-compatible) | SHA-256 (simplified) |
| **Protocol** | Wire protocol with encryption | CBOR messages (no encryption) |
| **Network** | QUIC transport | QUIC transport (planned) |
| **Storage** | SQLite + Git | SQLite + Git |
| **Features** | Full protocol implementation | Minimal subset |

---

## Data Flow

### Repository Initialization

```
User runs `rad init`
    ↓
Generate Ed25519 keypair (if not exists)
    ↓
Create identity document with DID:key
    ↓
Initialize git repository in storage
    ↓
Create bare repository with identity ref
    ↓
Push working copy to storage remote
    ↓
Create namespace refs (heads/*, patches/*)
    ↓
Store metadata in SQLite
```

### Patch Creation

```
User runs `rad patch create`
    ↓
Read current branch head
    ↓
Generate patch UUID
    ↓
Create patch entry in SQLite
    ↓
Create ref: refs/patches/<uuid>
    ↓
Announce to peers (planned)
```

### Peer Exchange (Planned)

```
Connect to seed nodes via QUIC
    ↓
Exchange inventory messages
    ↓
Update local peer list
    ↓
Fetch missing repositories
    ↓
Exchange ref announcements
    ↓
Update local refs
```

---

## Future Work

- [ ] QUIC transport for peer connections
- [ ] Encrypted protocol messages
- [ ] Background node process
- [ ] Real-time peer discovery
- [ ] Web of Trust integration
- [ ] Collaborative objects (issues, discussions)
