# rad config

View and manage configuration.

---

## Synopsis

```bash
rad config <SUBCOMMAND>
```

## Subcommands

| Subcommand | Description |
|------------|-------------|
| `show` | Display current configuration |
| `init` | Initialize default configuration |

---

## rad config show

Display the current configuration as formatted JSON.

### Synopsis

```bash
rad config show
```

### Examples

```bash
rad config show
```

### Output

```json
{
  "public_explorer": "https://app.radicle.example.com/nodes/$host/$rid$path",
  "preferred_seeds": [
    "z6MkrLMMsiPWUcNPHcRajuMi9mDfYckSoJyPwwnknocNYPm7@iris.radicle.network:8776",
    "z6Mkmqogy2qEM2ummccUthFEaaHvyYmYBYh3dbe9W4ebScxo@rosa.radicle.network:8776"
  ],
  "node": {
    "alias": "radicle-peer",
    "listen": ["127.0.0.1:8776"],
    "peers_type": "dynamic",
    "connect": [],
    "external_addresses": [],
    "network": "main",
    "log": "INFO"
  },
  "cli": {
    "hints": true
  }
}
```

---

## rad config init

Initialize the default configuration file.

### Synopsis

```bash
rad config init
```

### Examples

```bash
rad config init
```

### Output

```
Configuration initialized at: /home/user/.local/share/radicle/config.json
```

### Behavior

If a configuration file already exists, this command does nothing.

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

## Configuration Options

See [Configuration Reference](../configuration.md) for all available options.

---

## Environment Variables

| Variable | Description |
|----------|-------------|
| `RAD_HOME` | Override home directory |

---

## See Also

- [Configuration Reference](../configuration.md)
- [Installation](../installation.md)
