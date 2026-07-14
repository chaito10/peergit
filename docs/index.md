# Rad

**A minimal Radicle-inspired distributed code collaboration tool.**

---

Rad is a single-binary, peer-to-peer code collaboration tool inspired by [Heartwood](https://github.com/radicle-dev/heartwood), the reference implementation of the [Radicle](https://radicle.xyz) protocol. It provides decentralized repository management, cryptographic identity, patch workflows, and peer discovery without relying on centralized servers.

!!! info "Project Status"
    Rad is a minimal reference implementation designed for learning and experimentation. It is not intended for production use.

---

## Key Features

| Feature | Description |
|---------|-------------|
| **Ed25519 Identity** | Generate cryptographic identities with DID:key identifiers |
| **Repository Management** | Initialize, clone, push, and fetch repositories |
| **Peer Discovery** | Add, list, and manage known peers |
| **Patch Workflow** | Create, list, and merge patches through a review process |
| **SQLite Storage** | Local metadata storage for identities, repos, peers, and patches |
| **CBOR Protocol** | Compact binary serialization for network messages |
| **JSON Configuration** | Radicle-compatible configuration with sensible defaults |

---

## Architecture at a Glance

Rad flattens the multi-crate Heartwood architecture into a single executable:

```
src/main.rs
  mod crypto      -- Ed25519 keys, signing, verification
  mod identity    -- DID, project metadata, identity documents
  mod git         -- Git operations via libgit2
  mod storage     -- SQLite database
  mod protocol    -- CBOR network messages
  mod peer        -- Peer management
  mod config      -- JSON configuration
  mod home        -- Directory management
```

---

## Quick Example

```bash
# Generate an identity
rad id

# Initialize a repository
cd my-project
rad init --name "my-project" --description "A cool project"

# Check status
rad status

# Create a patch
rad patch create --title "Add new feature" --description "Implements X"

# List patches
rad patch list
```

---

## Navigation

Get started with the [Installation](installation.md) guide, or jump directly to the [Quick Start](quickstart.md) tutorial.

For complete command reference, see the [Commands](commands/index.md) section.

---

## License

Licensed under either of:

- Apache License, Version 2.0
- MIT License

at your option.
