# PeerGit

**P2P transport, discovery, identity and collaboration layer for Fossil repositories.**

PeerGit makes existing Fossil repositories directly discoverable and synchronizable over a decentralized peer-to-peer network. Fossil owns the repository state; PeerGit owns the networking.

---

## Features

- **Fossil Transport Adapter** -- Bridges Fossil's `--transport-command` with libp2p for P2P sync
- **Ed25519 Identity** -- Cryptographic identities with DID:key and libp2p PeerId
- **Kademlia DHT** -- Decentralized peer discovery
- **Request-Response Protocol** -- Length-prefixed Fossil xfer messages over TCP+Noise+Yamux
- **Web Dashboard** -- Embedded HTML/JS dashboard for peer and repo management (no Node/npm)
- **Single Binary** -- Zero runtime dependencies; no Node, Docker, or libgit2 required
- **Application Metadata DB** -- SQLite for identity, peers, and published repositories

---

## Install

### Pre-built Binaries

Download the latest release for your platform from [GitHub Releases](https://github.com/chaito10/peergit/releases):

| Platform | Archive |
|----------|---------|
| Windows x64 | `peergit-v0.1.0-x86_64-pc-windows-msvc.zip` |
| Linux x64 | `peergit-v0.1.0-x86_64-unknown-linux-gnu.tar.gz` |
| Linux ARM64 | `peergit-v0.1.0-aarch64-unknown-linux-gnu.tar.gz` |
| macOS x64 | `peergit-v0.1.0-x86_64-apple-darwin.tar.gz` |
| macOS ARM64 (M1/M2) | `peergit-v0.1.0-aarch64-apple-darwin.tar.gz` |

### Scoop (Windows)

```bash
scoop bucket add chaito10 https://github.com/chaito10/scoop-bucket
scoop install peergit
```

### From source

```bash
cargo install --path .
```

---

## Quick Start

```bash
# Initialize a node identity
peergit init

# Show identity
peergit identity

# Add a peer
peergit peer add <public-key> --alias alice --addresses /ip4/192.168.1.10/tcp/4001/p2p/<peer-id>

# Start the node (libp2p swarm + web dashboard on :3000)
peergit node start

# Publish a Fossil repository
peergit repo publish --path ./my-project --name my-project

# Sync with a remote via Fossil's transport-command
fossil sync --transport-command "peergit transport" /ip4/192.168.1.10/tcp/4001/p2p/<peer-id>
```

---

## Usage

```
peergit <COMMAND>

Commands:
  init              Initialize node identity and home directory
  identity          Show node identity (DID, PeerId, public key)
  peer              Manage known peers
    add             Add a peer with public key and address
    list            List known peers
  node              Node management
    start           Start the P2P node + web dashboard
    status          Show node status
  repo              Repository management
    list            List published repositories
    publish         Publish a local Fossil repository
    discover        Discover a repository by RID
    clone           Clone a published repository
  sync              Sync a repository (P2P or Fossil)
  config            Configuration management
  fossil            Pass through to fossil CLI
  transport         Transport adapter (called by fossil --transport-command)
```

---

## Architecture

PeerGit does **not** replace Fossil. It wraps it:

```
Fossil sync / push / pull
  |
  --transport-command
  |
  peergit transport
  |
  libp2p (TCP + Noise + Yamux)
  |
  Kademlia DHT (peer discovery)
  |
  Request-Response (/peergit/xfer/1.0)
  |
  Remote peer
  |
  fossil test-http (processes the request)
```

Fossil remains responsible for all repository synchronization. PeerGit is the network adapter.

---

## Web Dashboard

When the node is running, open http://localhost:3000 for a dashboard showing:

- Node status and identity
- Known peers (add, list)
- Published repositories
- Sync operations

Fossil's own web UI (wiki, tickets, timeline) runs separately via `fossil ui`.

---

## Configuration

PeerGit stores configuration in:

- **Linux/macOS**: `~/.local/share/peergit/config.json`
- **Windows**: `%LOCALAPPDATA%/peergit/config.json`

Override with `PEERGIT_HOME=/path/to/home`.

---

## Examples

See the [examples/](examples/) folder for detailed step-by-step guides:

- [Basic Setup](examples/basic-setup.md) -- Initialize, configure, start
- [Peer Discovery](examples/peer-discovery.md) -- Add peers, verify connectivity
- [Fossil Sync](examples/fossil-sync.md) -- Clone, sync, push over libp2p
- [Web Dashboard](examples/web-dashboard.md) -- Monitor nodes and repos
- [Multi-Node](examples/multi-node.md) -- 3-node network on one machine
- [Transport Command](examples/transport-command.md) -- Use as Fossil's transport

---

## Building

```bash
# Debug build
cargo build

# Release build (optimized, LTO, stripped)
cargo build --release
```

The release binary is located at `target/release/peergit`.

---

## License

PeerGit is distributed under the PeerGit Non-Commercial License v1.0.

- Personal use, education, research, open-source contributions, and academic use are permitted.
- Commercial use, SaaS offerings, paid products, and enterprise deployment for commercial advantage are not permitted.

For commercial licensing, please contact the project maintainer.
