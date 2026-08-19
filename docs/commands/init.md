# peergit init

Initialize node identity and home directory.

---

## Usage

```bash
peergit init
```

---

## Description

Creates a new Ed25519 keypair and initializes the PeerGit home directory with:

- Node identity (DID:key, PeerId, public key)
- Configuration file with defaults
- SQLite database schema

---

## Output

```
Node initialized at C:\Users\you\AppData\Local\peergit
DID:        did:key:z6Mk...
PeerId:     12D3KooW...
Public Key: 6C4X...
```

---

## What It Creates

```
$PEERGIT_HOME/
  config.json        # Default configuration
  peergit.db         # SQLite database
  keys/
    node             # Ed25519 secret key
```

---

## See Also

- [peergit identity](id.md) -- View identity after initialization
- [Configuration](../configuration.md) -- Customize the configuration
