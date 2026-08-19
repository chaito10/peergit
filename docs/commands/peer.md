# peergit peer

Manage known peers.

---

## peergit peer add

Add a peer with public key and address.

### Usage

```bash
peergit peer add <PUBLIC_KEY> --alias <NAME> --addresses <MULTIADDR>
```

### Arguments

| Argument | Description |
|----------|-------------|
| `<PUBLIC_KEY>` | Multibase-encoded Ed25519 public key |

### Options

| Option | Description |
|--------|-------------|
| `--alias <NAME>` | Human-readable alias for the peer |
| `--addresses <MULTIADDR>` | libp2p multiaddress(es) |

### Example

```bash
peergit peer add 6C4X... --alias bob \
  --addresses /ip4/192.168.1.20/tcp/4001/p2p/12D3KooW...
```

---

## peergit peer list

List known peers.

### Usage

```bash
peergit peer list
```

### Output

```
Known peers:
  bob
    PeerId:   12D3KooW...
    Addresses: /ip4/192.168.1.20/tcp/4001/p2p/12D3KooW...
    Added:     2026-08-19
```

---

## See Also

- [Peers Concept](../concepts/peers.md) -- Understanding peers
