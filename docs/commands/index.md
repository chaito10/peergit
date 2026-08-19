# Commands Reference

This section documents all available PeerGit commands.

---

## Overview

PeerGit provides a git-like CLI interface for P2P Fossil collaboration.

### Usage Pattern

```bash
peergit <command> [subcommand] [options]
```

### Global Options

| Option | Description |
|--------|-------------|
| `-h`, `--help` | Print help information |
| `-V`, `--version` | Print version information |

---

## Command Categories

### Identity

| Command | Description |
|---------|-------------|
| `peergit init` | Initialize node identity and home directory |
| `peergit identity` | Show node identity (DID, PeerId, public key) |

### Peer Management

| Command | Description |
|---------|-------------|
| `peergit peer add` | Add a peer with public key and address |
| `peergit peer list` | List known peers |

### Node Management

| Command | Description |
|---------|-------------|
| `peergit node start` | Start the P2P node + web dashboard |
| `peergit node status` | Show node status |

### Repository Management

| Command | Description |
|---------|-------------|
| `peergit repo list` | List published repositories |
| `peergit repo publish` | Publish a local Fossil repository |
| `peergit repo discover` | Discover a repository by RID |
| `peergit repo clone` | Clone a published repository |

### Synchronization

| Command | Description |
|---------|-------------|
| `peergit sync` | Sync a repository (P2P or Fossil) |

### Configuration

| Command | Description |
|---------|-------------|
| `peergit config show` | Show current configuration |
| `peergit config init` | Initialize default config |
| `peergit config get` | Get a config value |
| `peergit config set` | Set a config value |

### Fossil Passthrough

| Command | Description |
|---------|-------------|
| `peergit fossil` | Pass through to fossil CLI |

### Transport

| Command | Description |
|---------|-------------|
| `peergit transport` | Transport adapter (called by fossil `--transport-command`) |

---

## Exit Codes

| Code | Description |
|------|-------------|
| 0 | Success |
| 1 | General error |
| 2 | Invalid arguments |
| 3 | Fossil operation failed |
| 4 | Database error |
| 5 | Identity not found |

---

## Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `PEERGIT_HOME` | Override home directory | Platform-specific |
| `RUST_LOG` | Log level filter | `warn` |
