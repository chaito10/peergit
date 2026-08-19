# Example: Peer Discovery

Add peers and establish P2P connectivity.

## Step 1: Get Your PeerId

On each machine, run:
```bash
peergit identity
```

Note the `PeerId` and `Public Key` values.

## Step 2: Add a Peer (Node A)

```bash
peergit peer add <NODE_B_PUBLIC_KEY> \
  --alias bob \
  --addresses /ip4/192.168.1.20/tcp/4001/p2p/<NODE_B_PEER_ID>
```

## Step 3: List Known Peers

```bash
peergit peer list
```

Output:
```
Known peers:
  bob
    PeerId:   12D3KooW...
    Addresses: /ip4/192.168.1.20/tcp/4001/p2p/12D3KooW...
    Added:     2026-08-19
```

## Step 4: Verify Connectivity

Start the node:
```bash
peergit node start
```

Check status in another terminal:
```bash
peergit node status
```

You should see `Connected peers: 1` once the peer connects.

## Step 5: Add Peers via Dashboard

1. Open `http://localhost:3000`
2. Click the "Peers" tab
3. Enter the peer's public key and multiaddress
4. Click "Add Peer"

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

## Peer Persistence

Peers are stored in the SQLite database at:
- **Linux**: `~/.local/share/peergit/peergit.db`
- **macOS**: `~/Library/Application Support/peergit/peergit.db`
- **Windows**: `%LOCALAPPDATA%\peergit\peergit.db`

Peers survive node restarts.
