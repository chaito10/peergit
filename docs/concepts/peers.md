# Peers

PeerGit enables decentralized collaboration by connecting peers who exchange repository information over libp2p.

---

## Overview

A peer in PeerGit is another node identified by their Ed25519 public key. Peers can:

- Exchange Fossil xfer requests over libp2p
- Discover each other via Kademlia DHT
- Authenticate via Noise protocol handshake

---

## Peer Identity

Each peer is identified by their:

- **Public Key**: Multibase-encoded Ed25519 key
- **PeerId**: libp2p identifier derived from the public key
- **DID**: `did:key` identifier derived from the public key

### Aliases

Peers can have human-readable aliases:

```bash
peergit peer add <PUBLIC_KEY> --alias alice \
  --addresses /ip4/192.168.1.10/tcp/4001/p2p/<PEER_ID>
```

---

## Managing Peers

### Add a Peer

```bash
peergit peer add <PUBLIC_KEY> --alias <NAME> --addresses <MULTIADDR>
```

### List Peers

```bash
peergit peer list
```

Output:

```
Known peers:
  alice
    PeerId:   12D3KooW...
    Addresses: /ip4/192.168.1.10/tcp/4001/p2p/12D3KooW...
    Added:     2026-08-19
```

---

## Peer Database

### Schema

```sql
CREATE TABLE known_peers (
    public_key TEXT PRIMARY KEY,
    alias TEXT NOT NULL,
    addresses TEXT NOT NULL,
    added_at INTEGER NOT NULL
);
```

### Fields

| Field | Type | Description |
|-------|------|-------------|
| `public_key` | TEXT | Multibase-encoded public key |
| `alias` | TEXT | Human-readable name |
| `addresses` | TEXT | JSON array of multiaddresses |
| `added_at` | INTEGER | Unix timestamp when added |

---

## Peer Discovery

PeerGit uses two discovery mechanisms:

### Manual Discovery

Add peers explicitly with `peergit peer add`:

```bash
peergit peer add <PUBKEY> --alias bob \
  --addresses /ip4/192.168.1.20/tcp/4001/p2p/<BOB_PEER_ID>
```

### Kademlia DHT

Once connected to a peer, PeerGit uses the Kademlia DHT to discover other peers in the network. The DHT stores peer records keyed by their PeerId.

!!! info "DHT Bootstrap"
    To participate in DHT discovery, your node must be connected to at least one peer. Add a known peer first, then start the node.

---

## Connection Protocol

When PeerGit connects to a peer:

```
1. TCP handshake
    |
2. Noise handshake (XX pattern)
    |  - Authentication: Ed25519 signatures
    |  - Key exchange: X25519 Diffie-Hellman
    |  - Encryption: AES-256-GCM
    |
3. Yamux multiplexing negotiation
    |
4. Identify protocol exchange
    |  - Protocol version
    |  - Public key
    |  - Listen addresses
    |
5. Kademlia DHT protocol negotiation
    |
6. Request-Response protocol negotiation
    |  - /peergit/xfer/1.0
```

---

## Trust Model

### Current Model

Trust is manual:

1. Add a peer with `peergit peer add`
2. Manually verify their public key out-of-band
3. Trust requests from added peers

!!! warning "Security"
    Without active verification, the current model relies on manual key verification. Always verify public keys through a trusted channel.

### Future Enhancements

- [ ] Web of Trust integration
- [ ] Peer reputation system
- [ ] Automatic peer discovery via bootstrap nodes
