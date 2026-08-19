# Fossil Sync

PeerGit enables Fossil repository synchronization over libp2p.

---

## Overview

PeerGit integrates with Fossil's `--transport-command` flag to route synchronization traffic over libp2p instead of HTTP/TCP.

---

## How It Works

### The Transport Command Interface

Fossil supports custom transport commands:

```bash
fossil sync --transport-command "peergit transport" <REMOTE_URL>
```

Fossil calls PeerGit with:

```
peergit transport <url> <request_file> <reply_file>
```

PeerGit:

1. Reads the HTTP xfer request from `<request_file>`
2. Connects to the remote peer via libp2p
3. Sends the request over the `/peergit/xfer/1.0` protocol
4. Receives the response
5. Writes it to `<reply_file>`

---

## Request-Response Protocol

PeerGit uses the `/peergit/xfer/1.0` request-response protocol:

### Frame Format

```
[4 bytes: length (big-endian)] [payload: HTTP request/response]
```

- Maximum payload size: 100MB
- Timeout: 60 seconds
- Encoding: Length-prefixed binary framing

### Protocol Flow

```
Alice (sender)                    Bob (receiver)
     |                                 |
     |--- TCP + Noise handshake ------>|
     |                                 |
     |--- Request (Fossil xfer) ------>|
     |                                 |
     |                    fossil test-http processes request
     |                                 |
     |<-- Response (Fossil xfer) ------|
     |                                 |
     |--- Connection close ------------>|
```

---

## Sender Side (Alice)

Alice wants to sync with Bob:

```bash
# Start the node
peergit node start

# Sync using PeerGit as transport
fossil sync --transport-command "peergit transport" \
  /ip4/192.168.1.20/tcp/4001/p2p/<BOB_PEER_ID>
```

PeerGit:

1. Resolves Bob's PeerId from the multiaddress
2. Connects to Bob via TCP + Noise + Yamux
3. Sends the Fossil xfer request
4. Receives the response
5. Writes it to the reply file
6. Fossil processes the response

---

## Receiver Side (Bob)

Bob must have:

1. A running PeerGit node (`peergit node start`)
2. The repository published (`peergit repo publish`)
3. Fossil available to process the xfer request

When Alice connects:

1. PeerGit receives the inbound request
2. Writes it to a temporary file
3. Runs `fossil test-http <temp_file> <reply_file>`
4. Fossil processes the xfer against local repos
5. PeerGit reads the response and sends it back to Alice

---

## Multi-Address Formats

PeerGit supports standard libp2p multiaddresses:

```
# TCP + Noise (default)
/ip4/192.168.1.20/tcp/4001/p2p/12D3KooW...

# With DNS
/dns4/example.com/tcp/4001/p2p/12D3KooW...

# IPv6
/ip6/::1/tcp/4001/p2p/12D3KooW...
```

---

## Error Handling

| Error | Cause | Fix |
|-------|-------|-----|
| `Connection refused` | Remote node not running | Start peer's `peergit node start` |
| `Peer not found` | Wrong PeerId | Check with `peergit peer list` |
| `Repository not found` | Remote didn't publish the repo | Ask peer to run `peergit repo publish` |
| `Fossil not found` | Fossil binary not in PATH | Set `peergit config set fossil.fossil_path /path/to/fossil` |
| `Timeout` | Network issue | Check firewall, try different port |

---

## Performance

For large repositories:

- **Length-prefixed framing**: 4-byte header, up to 100MB per message
- **TCP transport**: Reliable, ordered delivery
- **Yamux multiplexing**: Multiple streams over one connection
- **60-second timeout**: Configurable for slow networks

For very large syncs (>100MB), consider splitting into smaller commits.

---

## Security

All communication over libp2p is encrypted using the Noise protocol:

- Transport encryption: Noise XX handshake pattern
- Peer authentication: Ed25519 signatures
- Key exchange: X25519 Diffie-Hellman
- Encryption: AES-256-GCM

No plaintext is sent over the network.

---

## See Also

- [Transport Command](../commands/transport.md) -- Low-level transport adapter
- [Architecture](../architecture.md) -- System design
- [Examples](https://github.com/chaito10/peergit/tree/main/examples/fossil-sync.md) -- Step-by-step sync guide
