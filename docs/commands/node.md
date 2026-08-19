# peergit node

Node management commands.

---

## peergit node start

Start the P2P node and web dashboard.

### Usage

```bash
peergit node start
```

### Description

Starts three components:

1. **libp2p swarm** -- Listens for inbound connections and manages the DHT
2. **Kademlia DHT** -- Decentralized peer discovery
3. **Web dashboard** -- HTTP server on the configured port (default: 3000)

Press Ctrl+C to stop.

### Output

```
Node started
  PeerId:    12D3KooW...
  Listening: /ip4/0.0.0.0/tcp/4001
  Dashboard: http://localhost:3000
```

---

## peergit node status

Show node status.

### Usage

```bash
peergit node status
```

### Output

```
Node Status:
  PeerId:        12D3KooW...
  DID:           did:key:z6Mk...
  Listening:     /ip4/0.0.0.0/tcp/4001
  Connected:     2 peers
  Published:     3 repos
```

---

## See Also

- [Web Dashboard](../concepts/web-dashboard.md) -- Dashboard features
- [Configuration](../configuration.md) -- Configure ports and addresses
