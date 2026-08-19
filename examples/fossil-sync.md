# Example: Fossil Sync

Use PeerGit to synchronize Fossil repositories over libp2p.

## Method 1: Direct Fossil Sync

PeerGit integrates with Fossil's `--transport-command` flag.

### Sender (Alice)

```bash
# Clone the repo locally first
fossil clone https://example.com/repo.fossil repo.fossil

# Create a working checkout
mkdir repo && cd repo
fossil open ../repo.fossil

# Make changes and commit
echo "Hello from Alice" > README.md
fossil add README.md
fossil commit -m "Initial commit"

# Sync using PeerGit as transport
fossil sync --transport-command "peergit transport" \
  /ip4/192.168.1.20/tcp/4001/p2p/<BOB_PEER_ID>
```

### Receiver (Bob)

Bob must have the repository published:
```bash
# Publish the repo so PeerGit knows about it
peergit repo publish --path ./repo --name my-project

# Start the node to listen for incoming sync requests
peergit node start
```

When Alice syncs, Bob's node:
1. Receives the request via libp2p
2. Runs `fossil test-http` to process the xfer
3. Sends the response back over libp2p

## Method 2: Using the P2P Layer

### Publish a Repository

```bash
# Initialize a Fossil repo
fossil init my-project.fossil

# Publish it through PeerGit
peergit repo publish --path ./my-project.fossil --name my-project
```

This:
1. Computes a SHA256-based Repository ID (RID)
2. Stores the mapping in the local database
3. Makes it discoverable by peers

### Discover a Repository

```bash
# Find a repo by RID
peergit repo discover --rid <REPO_SHA256>

# Or search by name
peergit repo list
```

### Clone via Fossil

```bash
# Clone using Fossil's transport-command
fossil clone --transport-command "peergit transport" \
  /ip4/192.168.1.10/tcp/4001/p2p/<ALICE_PEER_ID> \
  my-project.fossil
```

## Method 3: Automated Sync

Create a sync script:

```bash
#!/bin/bash
# sync.sh - Sync all published repos with a peer

PEER_ADDR="/ip4/192.168.1.20/tcp/4001/p2p/<PEER_ID>"

# List published repos and sync each
peergit repo list --json | jq -r '.[].path' | while read repo; do
  echo "Syncing $repo..."
  fossil sync --transport-command "peergit transport" "$PEER_ADDR"
done
```

## Transport Protocol Details

PeerGit uses the `/peergit/xfer/1.0` request-response protocol:

1. Fossil sends an HTTP-style request to `<url>/<path>/xfer`
2. PeerGit wraps it in a libp2p request-response message
3. The remote peer processes it with `fossil test-http`
4. The response is sent back over the same libp2p stream

Frame format:
```
[4 bytes: length (big-endian)] [payload: Fossil xfer HTTP request/response]
```

## Troubleshooting

**"Connection refused"**
- Ensure the remote peer's node is running
- Check the multiaddress is correct
- Verify firewall allows the port

**"Repository not found"**
- The remote peer must `peergit repo publish` first
- Verify the RID or name matches

**"Fossil error"**
- Ensure Fossil v2.25+ is installed
- Check `peergit config get fossil.fossil_path`
