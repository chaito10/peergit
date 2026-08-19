# Architecture

PeerGit is a P2P transport, discovery, identity and collaboration layer for Fossil repositories. Fossil owns all SCM state; PeerGit owns networking and discovery.

---

## Design Philosophy

PeerGit does **not** replace Fossil. It acts as a transport adapter, wrapping Fossil's existing `--transport-command` interface with libp2p networking. This means:

- Fossil handles all repository state, commits, branches, and synchronization logic
- PeerGit handles peer discovery, identity, encrypted transport, and the web dashboard
- No fork of Fossil is needed — PeerGit works with stock Fossil v2.25+

---

## Module Overview

```
src/
  main.rs            -- Binary entry point
  lib.rs             -- Module declarations
  crypto/            -- Ed25519 keys, multibase, DID:key, libp2p PeerId
  identity/          -- DID, RepositoryIdentity, Visibility
  config/            -- FossilP2pConfig (JSON)
  home/              -- XDG directory management, PEERGIT_HOME
  fossil/            -- Fossil CLI wrapper (subprocess calls)
  storage/           -- SQLite schema (identity, repos, peers, advertised repos)
  p2p/               -- libp2p behaviour, codec, transport
  repository/        -- FossilRepoManager, SHA256-based RID
  transport/         -- Fossil transport adapter (one-shot sender, receiver)
  web/               -- Embedded HTTP dashboard (tokio TcpListener)
  cli/               -- CLI commands (clap)
  error.rs           -- FossilP2pError enum
```

---

## Module Details

### crypto

Handles cryptographic operations:

- **Key generation**: Ed25519 keypairs via `ed25519-dalek`
- **Multibase encoding**: Base32Z encoding for public keys
- **DID:key**: W3C Decentralized Identifiers derived from public keys
- **libp2p PeerId**: Conversion from Ed25519 keys to libp2p PeerId
- **Signing**: Message signing for identity verification
- **Verification**: Signature verification for received data

### identity

Manages decentralized identities:

- **Did**: `did:key` identifiers derived from public keys
- **RepositoryIdentity**: Repository metadata (name, description, RID)
- **Visibility**: Public or private repository visibility

### config

JSON configuration with sensible defaults:

- **Node**: Alias, listen addresses
- **P2P**: Kademlia protocol name, bootstrap peers
- **Fossil**: Fossil binary path, web dashboard port

### home

XDG-style home directory management:

- **Directory creation**: Config, keys, storage directories
- **File paths**: Configuration, database, key file paths
- **PEERGIT_HOME**: Environment variable override

### fossil

Wraps the Fossil CLI via subprocess calls:

- **Repository operations**: init, clone, sync, status, add, commit
- **Web UI**: `fossil ui` for wiki, tickets, timeline
- **Transport support**: `fossil test-http` for processing xfer requests
- **Timeline**: `fossil timeline` for recent changes

### storage

SQLite database for persistent state:

- **Identity**: Node DID, public key, key file path
- **Repositories**: RID (SHA256), name, path, advertised status
- **Known peers**: Public key, alias, addresses, added timestamp

### p2p

libp2p networking stack:

- **behaviour.rs**: FossilP2pBehaviour combining Identify + Kademlia + Ping + RequestResponse
- **codec.rs**: FossilCodec implementing `async_trait` request/response Codec with length-prefixed framing
- **transport.rs**: TCP + Noise + Yamux transport builder

### repository

Fossil repository management:

- **FossilRepoManager**: List repos, compute RIDs
- **RID computation**: SHA256 hash of repo path + name

### transport

Fossil transport adapter (the core integration point):

- **run_transport**: One-shot transport command called by Fossil
- **run_receiver_request**: Process incoming xfer requests
- **URL resolution**: Parse multiaddresses, resolve peer IDs
- **Key management**: Load node keypair for signing

### web

Embedded web dashboard:

- **mod.rs**: HTTP server using tokio TcpListener
- **html.rs**: Embedded single-page dashboard (HTML/JS/CSS)
- **api.rs**: JSON API handlers (status, peers, repos, sync)

---

## Data Flow

### Sending a Sync Request (Alice)

```
fossil sync --transport-command "peergit transport" <BOB_ADDR>
    |
    Fossil writes HTTP xfer request to request_file
    |
    peergit transport <url> <request_file> <reply_file>
    |
    Read request from request_file
    |
    Resolve <url> to libp2p PeerId
    |
    Connect to Bob via TCP + Noise + Yamux
    |
    Send request over /peergit/xfer/1.0 protocol
    |
    Receive response
    |
    Write response to reply_file
    |
    Fossil reads reply_file and processes the response
```

### Receiving a Sync Request (Bob)

```
peergit node start
    |
    libp2p swarm listens on configured port
    |
    Receives inbound /peergit/xfer/1.0 request from Alice
    |
    run_receiver_request() processes the request
    |
    Writes request to temp file
    |
    Runs: fossil test-http <temp_file> <reply_file>
    |
    Fossil processes the xfer against local repos
    |
    Reads reply from reply_file
    |
    Sends response back to Alice over libp2p
```

---

## Network Stack

```
Application Layer:     Fossil (sync, push, pull)
                            |
Transport Adapter:     peergit transport (HTTP xfer <-> libp2p)
                            |
Protocol Layer:        Request-Response (/peergit/xfer/1.0)
                            |
Security Layer:        Noise (XX handshake, Curve25519, Ed25519 auth)
                            |
Transport Layer:       TCP + Yamux multiplexing
                            |
Discovery Layer:       Kademlia DHT + Identify protocol
```

---

## Comparison with Alternatives

| Aspect | PeerGit | Radicle Heartwood | Fossil built-in |
|--------|---------|-------------------|-----------------|
| **SCM** | Fossil | Git | Fossil |
| **Transport** | libp2p (TCP+Noise+Yamux) | QUIC | HTTP/TCP |
| **Discovery** | Kademlia DHT | Custom DHT | Centralized |
| **Identity** | Ed25519 + DID:key | Ed25519 + SHA-1 OID | None |
| **Encryption** | Noise (built-in) | QUIC (built-in) | Optional TLS |
| **Dashboard** | Embedded web UI | None | Built-in |
| **Binary** | Single binary | Multi-crate | N/A |
