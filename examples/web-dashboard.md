# Example: Web Dashboard

Use the embedded web dashboard to monitor your PeerGit node.

## Start the Node

```bash
peergit node start
```

The dashboard is available at `http://localhost:3000`.

## Dashboard Tabs

### Dashboard Tab

Shows:
- **Node Status**: Online/Offline, PeerId, DID
- **Listening Address**: The libp2p multiaddress
- **Connected Peers**: Number of active connections
- **Published Repos**: Number of repos available for sync

### Peers Tab

- **Add Peer**: Enter public key and multiaddress
- **Peer List**: Shows all known peers with status
- **Remove Peer**: Delete a peer from the known list

### Repos Tab

- **Published Repos**: List of repos this node shares
- **Add Repo**: Register a Fossil repo for P2P sync
- **Repo Details**: RID, path, name, added date

## API Endpoints

The dashboard uses a JSON API you can call directly:

```bash
# Get node status
curl http://localhost:3000/api/status

# List peers
curl http://localhost:3000/api/peers

# List repos
curl http://localhost:3000/api/repos

# Add a peer
curl -X POST http://localhost:3000/api/peers \
  -H "Content-Type: application/json" \
  -d '{"public_key":"...","addresses":["/ip4/.../tcp/.../p2p/..."],"alias":"bob"}'

# Trigger a sync
curl -X POST http://localhost:3000/api/sync \
  -H "Content-Type: application/json" \
  -d '{"peer_id":"..."}'
```

## Custom Port

```bash
# Change the dashboard port
peergit config set fossil.web_port 8080

# Restart to apply
peergit node start
# Now at http://localhost:8080
```

## Fossil's Web UI

PeerGit's dashboard is separate from Fossil's built-in web UI:

- **PeerGit dashboard** (port 3000): P2P networking, peers, repos
- **Fossil UI** (port 8080): Wiki, tickets, timeline, source browsing

Start Fossil's UI with:
```bash
fossil ui
```

Or in the background:
```bash
fossil server --port 8080
```
