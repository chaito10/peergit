# PeerGit Examples

Step-by-step examples for using PeerGit as a P2P transport layer for Fossil repositories.

## Examples

| Example | Description |
|---------|-------------|
| [basic-setup.md](basic-setup.md) | Initialize a node, check identity, start the P2P swarm |
| [peer-discovery.md](peer-discovery.md) | Add peers, verify connectivity, exchange identity |
| [fossil-sync.md](fossil-sync.md) | Clone, sync, and push Fossil repos over libp2p |
| [web-dashboard.md](web-dashboard.md) | Use the embedded web dashboard for monitoring |
| [multi-node.md](multi-node.md) | Set up a 3-node network on one machine |
| [transport-command.md](transport-command.md) | Use `peergit transport` as Fossil's `--transport-command` |

## Prerequisites

- PeerGit binary (`peergit`) in your PATH
- Fossil v2.25+ installed and accessible
- (Optional) Two or more terminals for multi-node examples

## Quick Start

```bash
# 1. Initialize your node identity
peergit init

# 2. Check your identity
peergit identity

# 3. Start the P2P node + web dashboard
peergit node start

# 4. Open the dashboard at http://localhost:3000
```
