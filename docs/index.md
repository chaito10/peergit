# PeerGit

**P2P transport, discovery, identity and collaboration layer for Fossil repositories.**

PeerGit makes existing Fossil repositories directly discoverable and synchronizable over a decentralized peer-to-peer network. Fossil owns the repository state; PeerGit owns the networking.

---

## Key Features

| Feature | Description |
|---------|-------------|
| **Fossil Transport Adapter** | Bridges Fossil's `--transport-command` with libp2p for P2P sync |
| **Ed25519 Identity** | Cryptographic identities with DID:key and libp2p PeerId |
| **Kademlia DHT** | Decentralized peer discovery |
| **Request-Response Protocol** | Length-prefixed Fossil xfer messages over TCP+Noise+Yamux |
| **Web Dashboard** | Embedded HTML/JS dashboard for peer and repo management |
| **Single Binary** | Zero runtime dependencies; no Node, Docker, or libgit2 required |
| **SQLite Storage** | Application metadata for peers, repos, and identity |

---

## Architecture at a Glance

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

---

## Quick Example

```bash
# Initialize a node identity
peergit init

# Show identity
peergit identity

# Add a peer
peergit peer add <public-key> --alias alice \
  --addresses /ip4/192.168.1.10/tcp/4001/p2p/<peer-id>

# Start the node (libp2p swarm + web dashboard on :3000)
peergit node start

# Sync a Fossil repo over libp2p
fossil sync --transport-command "peergit transport" \
  /ip4/192.168.1.10/tcp/4001/p2p/<peer-id>
```

---

## Navigation

Get started with the [Installation](installation.md) guide, or jump directly to the [Quick Start](quickstart.md) tutorial.

For complete command reference, see the [Commands](commands/index.md) section.

---

## License

PeerGit is distributed under the PeerGit Non-Commercial License v1.0.

- Personal use, education, research, open-source contributions, and academic use are permitted.
- Commercial use, SaaS offerings, paid products, and enterprise deployment for commercial advantage are not permitted.

For commercial licensing, please contact the project maintainer.
