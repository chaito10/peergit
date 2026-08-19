# peergit config

Configuration management commands.

---

## peergit config show

Show current configuration.

### Usage

```bash
peergit config show
```

### Output

```json
{
  "node": { "alias": "fossil-p2p-node" },
  "p2p": {
    "listen": ["/ip4/0.0.0.0/tcp/0"],
    "kad_protocol": "/peergit/kad/1.0"
  },
  "fossil": {
    "fossil_path": "fossil",
    "web_port": 3000
  }
}
```

---

## peergit config init

Initialize default configuration.

### Usage

```bash
peergit config init
```

---

## peergit config get

Get a config value.

### Usage

```bash
peergit config get <KEY>
```

### Example

```bash
peergit config get fossil.web_port
# Output: 3000
```

---

## peergit config set

Set a config value.

### Usage

```bash
peergit config set <KEY> <VALUE>
```

### Examples

```bash
peergit config set node.alias "alice-node"
peergit config set fossil.web_port 8080
peergit config set p2p.listen '["/ip4/0.0.0.0/tcp/4001"]'
```

---

## See Also

- [Configuration Reference](../configuration.md) -- Full configuration schema
