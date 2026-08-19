# Installation

This guide covers how to install PeerGit.

---

## Prerequisites

| Dependency | Minimum Version | Purpose |
|------------|-----------------|---------|
| [Fossil](https://fossil-scm.org/) | 2.25+ | Repository management |
| [Rust](https://rustup.rs/) | 1.75+ | Building from source (optional) |

!!! note "Fossil"
    PeerGit requires Fossil v2.25 or later for the `--transport-command` and `test-http` features. Install Fossil from [fossil-scm.org](https://fossil-scm.org/) or your package manager.

---

## Pre-built Binaries

Download the latest release from [GitHub Releases](https://github.com/chaito10/peergit/releases):

| Platform | Archive |
|----------|---------|
| Windows x64 | `peergit-v0.1.0-x86_64-pc-windows-msvc.zip` |
| Linux x64 | `peergit-v0.1.0-x86_64-unknown-linux-gnu.tar.gz` |
| Linux ARM64 | `peergit-v0.1.0-aarch64-unknown-linux-gnu.tar.gz` |
| macOS x64 | `peergit-v0.1.0-x86_64-apple-darwin.tar.gz` |
| macOS ARM64 (M1/M2) | `peergit-v0.1.0-aarch64-apple-darwin.tar.gz` |

### Windows (Scoop)

```bash
scoop bucket add chaito10 https://github.com/chaito10/scoop-bucket
scoop install peergit
```

### Linux / macOS

```bash
tar -xzf peergit-v0.1.0-*.tar.gz
chmod +x peergit
sudo mv peergit /usr/local/bin/
```

---

## Building from Source

### Clone the Repository

```bash
git clone https://github.com/chaito10/peergit.git
cd peergit
```

### Build

=== "Debug"

    ```bash
    cargo build
    ```

    The debug binary is located at `target/debug/peergit`.

=== "Release"

    ```bash
    cargo build --release
    ```

    The release binary is located at `target/release/peergit`.

!!! tip "Release Build Optimizations"
    The release build applies aggressive optimizations:
    
    - Link-Time Optimization (LTO) enabled
    - Symbols stripped
    - `opt-level = "z"` (optimize for size)
    - `codegen-units = 1` (better optimization)
    - `panic = "abort"` (smaller runtime)

### Install

```bash
cargo install --path .
```

This installs the `peergit` binary to `~/.cargo/bin/`.

---

## Verify Installation

```bash
peergit --version
```

Expected output:

```
peergit 0.1.0
```

---

## Home Directory

On first run, PeerGit creates a home directory to store configuration, keys, and data:

| Platform | Default Location |
|----------|------------------|
| Linux | `~/.local/share/peergit/` |
| macOS | `~/Library/Application Support/peergit/` |
| Windows | `%LOCALAPPDATA%/peergit/` |

Override with the `PEERGIT_HOME` environment variable:

```bash
export PEERGIT_HOME=/path/to/custom/home
```

### Directory Structure

```
$PEERGIT_HOME/
  config.json        # Configuration file
  peergit.db         # SQLite database (identity, peers, repos)
  keys/
    node             # Ed25519 secret key
```

---

## Next Steps

- [Quick Start](quickstart.md) -- Get up and running in minutes
- [Configuration](configuration.md) -- Customize your PeerGit installation
- [Commands Reference](commands/index.md) -- Complete command documentation
