# Example: Fossil Transport Command

Use `peergit transport` as Fossil's `--transport-command` for P2P synchronization.

## How It Works

Fossil supports custom transport commands via `--transport-command`. PeerGit acts as the transport adapter:

```
fossil sync --transport-command "peergit transport" <REMOTE_URL>
```

Fossil calls PeerGit with:
```
peergit transport <url> <request_file> <reply_file>
```

PeerGit:
1. Reads the HTTP request from `<request_file>`
2. Connects to the remote peer via libp2p
3. Sends the request over the `/peergit/xfer/1.0` protocol
4. Receives the response
5. Writes it to `<reply_file>`

## Basic Usage

### Clone a Remote Repository

```bash
fossil clone --transport-command "peergit transport" \
  /ip4/192.168.1.10/tcp/4001/p2p/<PEER_ID> \
  remote-repo.fossil
```

### Sync an Existing Checkout

```bash
cd my-project/
fossil sync --transport-command "peergit transport" \
  /ip4/192.168.1.10/tcp/4001/p2p/<PEER_ID>
```

### Push Changes

```bash
fossil push --transport-command "peergit transport" \
  /ip4/192.168.1.10/tcp/4001/p2p/<PEER_ID>
```

## URL Format

The URL after `--transport-command` is a libp2p multiaddress:

```
/ip4/192.168.1.10/tcp/4001/p2p/12D3KooW...
```

Or with DNS:
```
/dns4/myhost.example.com/tcp/4001/p2p/12D3KooW...
```

## Automating with Git-like Config

Set the transport command globally so you don't have to type it each time:

```bash
# In your Fossil global config
fossil settings transport-command "peergit transport"
```

Or per-repository:

```bash
fossil settings --local transport-command "peergit transport"
```

Now you can simply run:
```bash
fossil sync /ip4/192.168.1.10/tcp/4001/p2p/<PEER_ID>
```

## Multi-Peer Sync

Sync with multiple peers in sequence:

```bash
#!/bin/bash
# sync-all.sh

PEERS=(
  "/ip4/192.168.1.10/tcp/4001/p2p/12D3KooW_Alice..."
  "/ip4/192.168.1.20/tcp/4001/p2p/12D3KooW_Bob..."
  "/ip4/192.168.1.30/tcp/4001/p2p/12D3KooW_Carol..."
)

for peer in "${PEERS[@]}"; do
  echo "Syncing with $peer..."
  fossil sync --transport-command "peergit transport" "$peer"
done
```

## Error Handling

Common errors and fixes:

| Error | Cause | Fix |
|-------|-------|-----|
| `Connection refused` | Remote node not running | Start peer's `peergit node start` |
| `Peer not found` | Wrong PeerId | Check with `peergit peer list` |
| `Repository not found` | Remote didn't publish the repo | Ask peer to run `peergit repo publish` |
| `Fossil not found` | Fossil binary not in PATH | Set `peergit config set fossil.fossil_path /path/to/fossil` |
| `Timeout` | Network issue | Check firewall, try different port |

## Security

All communication over libp2p is encrypted using **Noise** protocol (XX handshake pattern):

- Transport encryption: Noise with Curve25519
- Peer authentication: Ed25519 signatures
- Key exchange: X25519 Diffie-Hellman

No plaintext is sent over the network.

## Performance

For large repositories, PeerGit uses:

- **Length-prefixed framing**: 4-byte header, up to 100MB per message
- **TCP transport**: Reliable, ordered delivery
- **Yamux multiplexing**: Multiple streams over one connection
- **60-second timeout**: Configurable for slow networks

For very large syncs (>100MB), consider splitting into smaller commits.
