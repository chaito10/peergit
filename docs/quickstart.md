# Quick Start

This guide walks you through your first PeerGit session: initializing a node, adding a peer, and syncing a Fossil repository over libp2p.

---

## Step 1: Initialize

```bash
peergit init
```

Output:

```
Node initialized at C:\Users\you\AppData\Local\peergit
DID:        did:key:z6Mk...
PeerId:     12D3KooW...
Public Key: 6C4X...
```

!!! info "What happened?"
    PeerGit generated a new Ed25519 keypair, created a DID:key identity, and stored configuration in your platform's data directory.

---

## Step 2: Check Identity

```bash
peergit identity
```

Shows your node's DID, PeerId, public key, and key file path.

---

## Step 3: Add a Peer

```bash
peergit peer add <BOB_PUBLIC_KEY> \
  --alias bob \
  --addresses /ip4/192.168.1.20/tcp/4001/p2p/<BOB_PEER_ID>
```

You need Bob's public key and PeerId (ask Bob to run `peergit identity`).

---

## Step 4: Start the Node

```bash
peergit node start
```

This starts:

1. **libp2p swarm** on a random TCP port
2. **Kademlia DHT** for peer discovery
3. **Web dashboard** on `http://localhost:3000`

Open the dashboard to see node status, peers, and published repos.

---

## Step 5: Publish a Fossil Repository

In another terminal:

```bash
# Create a Fossil repo (if you don't have one)
fossil init my-project.fossil

# Publish it through PeerGit
peergit repo publish --path ./my-project.fossil --name my-project
```

---

## Step 6: Sync with a Remote

```bash
fossil sync --transport-command "peergit transport" \
  /ip4/192.168.1.20/tcp/4001/p2p/<BOB_PEER_ID>
```

This tells Fossil to use PeerGit as the transport layer, sending the sync request over libp2p to Bob.

---

## Step 7: Check Status

```bash
peergit node status
```

Shows connected peers, published repos, and node identity.

---

## Summary

You have learned how to:

1. **Initialize** a node identity with `peergit init`
2. **Verify** identity with `peergit identity`
3. **Add** peers with `peergit peer add`
4. **Start** the P2P node with `peergit node start`
5. **Publish** a Fossil repository with `peergit repo publish`
6. **Sync** over libp2p with `fossil sync --transport-command`
7. **Monitor** with the web dashboard at `http://localhost:3000`

---

## Next Steps

- [Commands Reference](commands/index.md) -- Complete command documentation
- [Concepts](concepts/identity.md) -- Understand PeerGit's core concepts
- [Configuration](configuration.md) -- Customize your installation
- [Architecture](architecture.md) -- Understand the system design
- [Examples](https://github.com/chaito10/peergit/tree/main/examples) -- Step-by-step examples
