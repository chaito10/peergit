# Installation

This guide covers how to install Rad from source.

---

## Prerequisites

Before building Rad, ensure you have the following installed:

| Dependency | Minimum Version | Purpose |
|------------|-----------------|---------|
| [Rust](https://rustup.rs/) | 1.75+ | Compiler toolchain |
| [Git](https://git-scm.com/) | 2.0+ | Version control |
| [CMake](https://cmake.org/) | 3.0+ | Building libgit2 |
| [OpenSSL](https://www.openssl.org/) | 1.1+ | TLS support |

!!! note "Windows Users"
    On Windows, ensure you have the [Microsoft C++ Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/) or Visual Studio with the C++ workload installed.

---

## Building from Source

### Clone the Repository

```bash
git clone https://github.com/example/rad.git
cd rad
```

### Build

=== "Debug"

    ```bash
    cargo build
    ```

    The debug binary is located at `target/debug/rad`.

=== "Release"

    ```bash
    cargo build --release
    ```

    The release binary is located at `target/release/rad`.

!!! tip "Release Build Optimizations"
    The release build applies aggressive optimizations for small binary size:
    
    - Link-Time Optimization (LTO) enabled
    - Symbols stripped
    - `opt-level = "z"` (optimize for size)
    - `codegen-units = 1` (better optimization)
    - `panic = "abort"` (smaller runtime)

### Install

```bash
cargo install --path .
```

This installs the `rad` binary to `~/.cargo/bin/`.

---

## Verify Installation

```bash
rad --version
```

Expected output:

```
rad 0.1.0
```

---

## Home Directory

On first run, Rad creates a home directory to store configuration, keys, and data:

| Platform | Default Location |
|----------|------------------|
| Linux | `~/.local/share/radicle/` |
| macOS | `~/Library/Application Support/radicle/` |
| Windows | `%LOCALAPPDATA%/radicle/` |

Override with the `RAD_HOME` environment variable:

```bash
export RAD_HOME=/path/to/custom/home
```

### Directory Structure

```
$RAD_HOME/
  config.json        # Configuration file
  storage/           # Git repositories (bare)
    <rid>/           # One directory per repository
  keys/
    radicle          # Ed25519 secret key (hex)
    radicle.pub      # Ed25519 public key (hex)
  node/              # Node runtime data
  cobs/              # Collaborative objects cache
  node.db            # SQLite database
```

---

## Next Steps

- [Quick Start](quickstart.md) -- Get up and running in minutes
- [Configuration](configuration.md) -- Customize your Rad installation
- [Commands Reference](commands/index.md) -- Complete command documentation
