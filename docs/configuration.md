# Configuration

Rad uses a JSON configuration file stored in your home directory.

---

## Configuration File Location

| Platform | Path |
|----------|------|
| Linux | `~/.local/share/radicle/config.json` |
| macOS | `~/Library/Application Support/radicle/config.json` |
| Windows | `%LOCALAPPDATA%/radicle/config.json` |

Override with `RAD_HOME`:

```bash
export RAD_HOME=/path/to/custom/home
```

---

## Viewing Configuration

```bash
rad config show
```

This displays the full configuration as formatted JSON.

---

## Configuration Schema

### Top-Level Fields

```json
{
  "public_explorer": "https://app.radicle.example.com/nodes/$host/$rid$path",
  "preferred_seeds": [...],
  "node": { ... },
  "cli": { ... }
}
```

### Public Explorer

URL template for browsing repositories on the web:

```json
{
  "public_explorer": "https://app.radicle.example.com/nodes/$host/$rid$path"
}
```

| Variable | Description |
|----------|-------------|
| `$host` | Node hostname |
| `$rid` | Repository ID |
| `$path` | Repository path |

### Preferred Seeds

List of seed node addresses for network connectivity:

```json
{
  "preferred_seeds": [
    "z6MkrLMMsiPWUcNPHcRajuMi9mDfYckSoJyPwwnknocNYPm7@iris.radicle.network:8776",
    "z6Mkmqogy2qEM2ummccUthFEaaHvyYmYBYh3dbe9W4ebScxo@rosa.radicle.network:8776"
  ]
}
```

Each entry follows the format: `<node_id>@<address>:<port>`

### Node Configuration

```json
{
  "node": {
    "alias": "radicle-peer",
    "listen": ["127.0.0.1:8776"],
    "peers_type": "dynamic",
    "connect": [],
    "external_addresses": [],
    "network": "main",
    "log": "INFO"
  }
}
```

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `alias` | string | `"radicle-peer"` | Human-readable node name |
| `listen` | array | `["127.0.0.1:8776"]` | Socket addresses to listen on |
| `peers_type` | string | `"dynamic"` | Peer management mode (`static` or `dynamic`) |
| `connect` | array | `[]` | Persistent peer connections |
| `external_addresses` | array | `[]` | Public addresses advertised to other nodes |
| `network` | string | `"main"` | Network identifier (`main` or `test`) |
| `log` | string | `"INFO"` | Log level (`TRACE`, `DEBUG`, `INFO`, `WARN`, `ERROR`) |

### CLI Configuration

```json
{
  "cli": {
    "hints": true
  }
}
```

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `hints` | boolean | `true` | Show helpful hints in CLI output |

---

## Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `RAD_HOME` | Override home directory | Platform-specific |
| `RUST_LOG` | Log level filter | `warn` |

---

## Modifying Configuration

Currently, configuration is managed by editing the JSON file directly. Use `rad config show` to view the current configuration, then edit the file with your preferred editor.

!!! warning "Backup Your Configuration"
    Before editing, make a backup of your configuration file:

    ```bash
    cp ~/.local/share/radicle/config.json ~/.local/share/radicle/config.json.bak
    ```

---

## Default Seed Nodes

Rad ships with two default seed nodes for the main network:

| Alias | Node ID | Address |
|-------|---------|---------|
| Iris | `z6MkrLMMsiPWUcNPHcRajuMi9mDfYckSoJyPwwnknocNYPm7` | `iris.radicle.network:8776` |
| Rosa | `z6Mkmqogy2qEM2ummccUthFEaaHvyYmYBYh3dbe9W4ebScxo` | `rosa.radicle.network:8776` |
