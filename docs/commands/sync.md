# peergit sync

Sync a repository over P2P or Fossil.

---

## Usage

```bash
peergit sync --path <REPO_PATH> --peer <PEER_ID>
```

---

## Description

Syncs a local Fossil repository with a remote peer via the libp2p transport layer.

This is equivalent to running:

```bash
fossil sync --transport-command "peergit transport" <PEER_ADDR>
```

---

## Options

| Option | Description |
|--------|-------------|
| `--path <PATH>` | Path to the local Fossil repository |
| `--peer <PEER_ID>` | PeerId or multiaddress of the remote peer |

---

## Example

```bash
peergit sync --path ./my-project.fossil --peer /ip4/192.168.1.20/tcp/4001/p2p/12D3KooW...
```

---

## See Also

- [Fossil Sync Concept](../concepts/fossil-sync.md) -- How Fossil sync works over libp2p
- [Transport Command](transport.md) -- Low-level transport adapter
