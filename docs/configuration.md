# Configuration

PeerGit uses a JSON configuration file stored in your home directory.

---

## Configuration File Location

| Platform | Path |
|----------|------|
| Linux | `~/.local/share/peergit/config.json` |
| macOS | `~/Library/Application Support/peergit/config.json` |
| Windows | `%LOCALAPPDATA%/peergit/config.json` |

Override with `PEERGIT_HOME`:

```bash
export PEERGIT_HOME=/path/to/custom/home
```

---

## Viewing Configuration

```bash
peergit config show
```

This displays the full configuration as formatted JSON.

---

## Configuration Schema

### Top-Level Fields

```json
{
  "node": { ... },
  "p2p": { ... },
  "fossil": { ... }
}
```

### Node Configuration

```json
{
  "node": {
    "alias": "fossil-p2p-node"
  }
}
```

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `alias` | string | `"fossil-p2p-node"` | Human-readable node name |

### P2P Configuration

```json
{
  "p2p": {
    "listen": ["/ip4/0.0.0.0/tcp/0"],
    "kad_protocol": "/peergit/kad/1.0"
  }
}
```

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `listen` | array | `["/ip4/0.0.0.0/tcp/0"]` | Multiaddresses to listen on (port 0 = random) |
| `kad_protocol` | string | `"/peergit/kad/1.0"` | Kademlia protocol name |

### Fossil Configuration

```json
{
  "fossil": {
    "fossil_path": "fossil",
    "web_port": 3000
  }
}
```

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `fossil_path` | string | `"fossil"` | Path to Fossil binary |
| `web_port` | integer | `3000` | Web dashboard port |

---

## Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `PEERGIT_HOME` | Override home directory | Platform-specific |
| `RUST_LOG` | Log level filter | `warn` |

---

## Modifying Configuration

Use the `peergit config` commands:

```bash
# Set a value
peergit config set node.alias "alice-node"
peergit config set fossil.web_port 8080
peergit config set p2p.listen '["/ip4/0.0.0.0/tcp/4001"]'

# Get a value
peergit config get node.alias

# Show full config
peergit config show
```

!!! warning "Backup Your Configuration"
    Before editing manually, make a backup:

    ```bash
    cp ~/.local/share/peergit/config.json ~/.local/share/peergit/config.json.bak
    ```

---

## Listening Addresses

PeerGit uses libp2p multiaddresses for listening:

```
# Listen on all interfaces, random port
/ip4/0.0.0.0/tcp/0

# Listen on specific port
/ip4/0.0.0.0/tcp/4001

# Listen on localhost only
/ip4/127.0.0.1/tcp/4001

# IPv6
/ip6/::1/tcp/4001
```

---

## Web Dashboard Port

The web dashboard runs on the port configured in `fossil.web_port`:

```bash
# Change from default 3000 to 8080
peergit config set fossil.web_port 8080

# Restart to apply
peergit node start
# Dashboard now at http://localhost:8080
```

---

## Database

PeerGit stores all state in a SQLite database at:

- **Linux**: `~/.local/share/peergit/peergit.db`
- **macOS**: `~/Library/Application Support/peergit/peergit.db`
- **Windows**: `%LOCALAPPDATA%\peergit\peergit.db`

The database contains:

| Table | Purpose |
|-------|---------|
| `identity` | Node DID, public key, key file path |
| `repositories` | RID, name, path, advertised status |
| `known_peers` | Public key, alias, addresses, added timestamp |
| `advertised_repos` | Repos shared with the P2P network |
