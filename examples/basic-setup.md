# Example: Basic Setup

Initialize a PeerGit node and verify it works.

## Step 1: Initialize

```bash
peergit init
```

This creates:
- A new Ed25519 keypair
- A DID:key identity
- Configuration in your platform's data directory

Output:
```
Node initialized at C:\Users\you\AppData\Local\peergit
DID:        did:key:z6Mk...
PeerId:     12D3KooW...
Public Key: 6C4X...
```

## Step 2: Verify Identity

```bash
peergit identity
```

Shows your node's:
- DID (Decentralized Identifier)
- PeerId (libp2p identifier)
- Public key (multibase encoded)
- Key file path

## Step 3: Check Configuration

```bash
peergit config show
```

Default configuration:
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

## Step 4: Start the Node

```bash
peergit node start
```

This starts:
1. **libp2p swarm** on a random TCP port (or configured port)
2. **Kademlia DHT** for peer discovery
3. **Web dashboard** on `http://localhost:3000`

Press Ctrl+C to stop.

## Step 5: Check Status

In another terminal:
```bash
peergit node status
```

Shows:
- Node PeerId
- Listening addresses
- Connected peers count
- Published repositories

## Configuration Overrides

Set custom values:

```bash
# Change the web dashboard port
peergit config set fossil.web_port 8080

# Change the listening address
peergit config set p2p.listen '["/ip4/0.0.0.0/tcp/4001"]'

# Change the node alias
peergit config set node.alias "alice-node"

# Use a custom home directory
export PEERGIT_HOME=/home/alice/.peergit
peergit init
```
