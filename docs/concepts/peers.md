# Peers

Rad enables decentralized collaboration by connecting peers who exchange repository information.

---

## Overview

A peer in Rad is another user identified by their public key. Peers can:

- Exchange repository announcements
- Share inventory (list of repositories)
- Sync branches and patches

---

## Peer Identity

Each peer is identified by their Ed25519 public key:

```
z6MkrLMMsiPWUcNPHcRajuMi9mDfYckSoJyPwwnknocNYPm7
```

### Aliases

Peers can have human-readable aliases:

```bash
rad peer add z6MkrLMMsiPWUcNPHcRajuMi9mDfYckSoJyPwwnknocNYPm7 --alias alice
```

---

## Managing Peers

### Add a Peer

```bash
rad peer add <public_key> --alias <name>
```

### List Peers

```bash
rad peer list
```

Output:

```
Public Key                              Alias      Added
----------------------------------------------------------------
z6MkrLMMsiPWUcNPHcRajuMi9mDfYckSo...  alice      2024-01-15
z6Mkmqogy2qEM2ummccUthFEaaHvyYmY...  bob        2024-01-16
```

### Remove a Peer

```bash
rad peer remove <public_key>
```

---

## Peer Database

### Schema

```sql
CREATE TABLE peers (
    public_key TEXT PRIMARY KEY,
    alias TEXT,
    node_id TEXT,
    added_at INTEGER NOT NULL
);
```

### Fields

| Field | Type | Description |
|-------|------|-------------|
| `public_key` | TEXT | Multibase-encoded public key |
| `alias` | TEXT | Human-readable name |
| `node_id` | TEXT | Node identifier (planned) |
| `added_at` | INTEGER | Unix timestamp when added |

---

## Seed Nodes

Seed nodes are well-known peers that help with peer discovery:

### Default Seeds

```json
{
  "preferred_seeds": [
    "z6MkrLMMsiPWUcNPHcRajuMi9mDfYckSoJyPwwnknocNYPm7@iris.radicle.network:8776",
    "z6Mkmqogy2qEM2ummccUthFEaaHvyYmYBYh3dbe9W4ebScxo@rosa.radicle.network:8776"
  ]
}
```

### Adding Seeds

Seeds are listed in the configuration file:

```bash
rad config show
```

Edit the `preferred_seeds` array to add or remove seeds.

---

## Protocol Messages

Peers exchange messages to share repository state:

### Announcement

An announcement proves ownership of a repository:

```json
{
  "type": "announcement",
  "rid": "1ca78dfe...",
  "refs": {
    "refs/heads/main": "52680c8de2dca294fa2482e8ef725a4cc1bb362c"
  },
  "timestamp": 1705312800,
  "signature": "...",
  "public_key": "z6Mk..."
}
```

### Inventory

An inventory message lists all repositories a peer has:

```json
{
  "type": "inventory",
  "repositories": [
    {
      "rid": "1ca78dfe...",
      "name": "my-project",
      "refs": {
        "refs/heads/main": "52680c8de2dca294fa2482e8ef725a4cc1bb362c"
      }
    }
  ],
  "timestamp": 1705312800,
  "signature": "...",
  "public_key": "z6Mk..."
}
```

---

## Peer Exchange Flow

!!! info "Planned Feature"
    The following describes the planned peer exchange protocol. Currently, Rad only stores peers locally without active exchange.

### 1. Connect to Seed

```
Connect to seed node via QUIC
    ↓
Exchange ping/pong
    ↓
Connected
```

### 2. Exchange Inventory

```
Send inventory message
    ↓
Receive peer's inventory
    ↓
Compare repositories
    ↓
Identify missing repos
```

### 3. Sync Repositories

```
Fetch missing repositories
    ↓
Update local refs
    ↓
Exchange patches
```

---

## Trust Model

### Web of Trust (Planned)

Rad plans to implement a web of trust where:

- Peers can vouch for other peers
- Trust is transitive through the graph
- Reputation is computed from trust relationships

### Current Model

Currently, trust is manual:

1. Add a peer with `rad peer add`
2. Manually verify their public key out-of-band
3. Trust announcements from added peers

!!! warning "Security"
    Without active verification, the current model relies on manual key verification. Always verify public keys through a trusted channel.

---

## Future Enhancements

- [ ] QUIC transport for peer connections
- [ ] Automatic peer discovery via seeds
- [ ] Encrypted protocol messages
- [ ] Web of trust integration
- [ ] Peer reputation system
