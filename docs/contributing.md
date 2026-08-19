# Contributing

Thank you for your interest in contributing to PeerGit!

---

## Overview

PeerGit is a P2P transport layer for Fossil repositories. Contributions are welcome.

---

## Development Setup

### Prerequisites

- Rust 1.75+
- Fossil 2.25+
- Git

### Clone and Build

```bash
git clone https://github.com/chaito10/peergit.git
cd peergit
cargo build
```

### Run Tests

```bash
cargo test
```

---

## Code Style

### Formatting

```bash
cargo fmt
```

### Linting

```bash
cargo clippy -- -D warnings
```

### Commit Messages

Use conventional commit format:

```
feat: Add new feature
fix: Fix bug in X
docs: Update documentation
refactor: Simplify Y
test: Add tests for Z
```

---

## Project Structure

```
src/
  main.rs            -- Binary entry point
  lib.rs             -- Module declarations
  crypto/            -- Ed25519 keys, multibase, DID:key
  identity/          -- DID, RepositoryIdentity, Visibility
  config/            -- FossilP2pConfig (JSON)
  home/              -- XDG directory management
  fossil/            -- Fossil CLI wrapper (subprocess)
  storage/           -- SQLite schema
  p2p/               -- libp2p behaviour, codec, transport
  repository/        -- FossilRepoManager, RID computation
  transport/         -- Fossil transport adapter
  web/               -- Embedded HTTP dashboard
  cli/               -- CLI commands (clap)
  error.rs           -- FossilP2pError enum
```

When adding features:

1. Keep code in the appropriate module
2. Add CLI commands in `cli/commands.rs`
3. Update documentation if needed

---

## Testing

### Unit Tests

Add unit tests within the source file:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_generation() {
        let keypair = Keypair::generate();
        assert_eq!(keypair.public.to_bytes().len(), 32);
    }
}
```

### Integration Tests

Test CLI commands manually:

```bash
# Test init flow
peergit init
peergit identity
peergit node status

# Test peer flow
peergit peer add <pubkey> --alias test --addresses /ip4/127.0.0.1/tcp/4001/p2p/<peer-id>
peergit peer list
```

---

## Architecture Principles

1. **Fossil owns SCM** -- PeerGit never replaces Fossil's repository management
2. **libp2p owns networking** -- Use standard libp2p protocols, don't reinvent transport
3. **Kademlia owns discovery** -- Use Kademlia DHT for peer discovery
4. **Single binary** -- No runtime dependencies, no Node/npm/Docker
5. **Rust-first** -- All code in Rust, no C dependencies beyond SQLite

---

## Documentation

Documentation uses MkDocs with Material theme:

```bash
pip install mkdocs-material
mkdocs serve
```

Open http://localhost:8000 to preview.

### Writing Documentation

- Use clear, concise language
- Include examples for all commands
- Use admonitions for important notes

---

## Pull Requests

### Process

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Run `cargo fmt` and `cargo clippy`
5. Run `cargo test`
6. Update documentation if needed
7. Submit a pull request

### PR Title Format

```
feat: Add new feature
fix: Fix bug in X
docs: Update documentation
```

### PR Description

Include:

- What changed and why
- How to test the changes
- Any breaking changes

---

## Reporting Issues

Use GitHub Issues for bug reports and feature requests.

### Bug Report Template

```markdown
**Describe the bug**
A clear description of what the bug is.

**To reproduce**
Steps to reproduce the behavior.

**Expected behavior**
What you expected to happen.

**Environment**
- OS: [e.g., Windows 11]
- Rust version: [e.g., 1.75]
- PeerGit version: [e.g., 0.1.0]
- Fossil version: [e.g., 2.28]
```

---

## License

By contributing, you agree that your contributions will be licensed under the same license as the project.

---

## Questions?

Open a GitHub Issue or Discussion for questions about contributing.
