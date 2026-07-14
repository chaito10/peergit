# Rad

**A minimal Radicle-inspired distributed code collaboration tool.**

Rad is a single-binary, peer-to-peer code collaboration tool inspired by [Heartwood](https://github.com/radicle-dev/heartwood). It provides decentralized repository management, identity, patch workflows, and peer discovery without relying on centralized servers.

---

## Features

- **Ed25519 Identity** -- Generate cryptographic identities with DID:key identifiers
- **Repository Management** -- Initialize, clone, push, and fetch repositories
- **Peer Discovery** -- Add, list, and manage known peers
- **Patch Workflow** -- Create, list, and merge patches through a review process
- **SQLite Storage** -- Local metadata storage for identities, repos, peers, and patches
- **CBOR Protocol** -- Compact binary serialization for network messages
- **JSON Configuration** -- Radicle-compatible configuration with sensible defaults

---

## Quick Start

```bash
# Install
cargo install --path .

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

## Commands

| Command | Description |
|---------|-------------|
| `rad init` | Initialize a Radicle repository |
| `rad clone` | Clone a repository from storage |
| `rad push` | Push changes to a remote |
| `rad fetch` | Fetch changes from a remote |
| `rad peer add` | Add a known peer |
| `rad peer list` | List known peers |
| `rad id` | Show identity information |
| `rad patch create` | Create a new patch |
| `rad patch list` | List patches |
| `rad patch merge` | Merge a patch |
| `rad config show` | Show configuration |
| `rad status` | Show repository status |
| `rad sync` | Sync repositories |

---

## Building

```bash
# Debug build
cargo build

# Release build (optimized)
cargo build --release
```

The release binary is located at `target/release/rad`.

---

## Configuration

Rad stores configuration in:

- **Linux/macOS**: `~/.local/share/radicle/config.json`
- **Windows**: `%LOCALAPPDATA%/radicle/config.json`

Override with `RAD_HOME=/path/to/home`.

See [Configuration Reference](docs/configuration.md) for all options.

---

## Architecture

Rad flattens the multi-crate Heartwood architecture into a single executable with logical modules:

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

See [Architecture Guide](docs/architecture.md) for details.

---

## Documentation

Full documentation is available in the `docs/` directory and can be served locally:

```bash
pip install mkdocs-material
mkdocs serve
```

Then open http://localhost:8000.

---

## License

Licensed under either of:

- Apache License, Version 2.0
- MIT License

at your option.
