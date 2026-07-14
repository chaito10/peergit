# Contributing

Thank you for your interest in contributing to Rad!

---

## Overview

Rad is a minimal reference implementation of the Radicle protocol. Contributions are welcome, but please keep in mind:

- This is a learning tool, not a production system
- Keep changes simple and focused
- Follow existing code style

---

## Development Setup

### Prerequisites

- Rust 1.75+
- Git
- CMake
- OpenSSL

### Clone and Build

```bash
git clone https://github.com/example/rad.git
cd rad
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

All code lives in `src/main.rs` as internal modules:

```
src/main.rs
  mod crypto      -- Ed25519 operations
  mod identity    -- DID and identity documents
  mod git         -- Git operations
  mod storage     -- SQLite database
  mod protocol    -- CBOR messages
  mod peer        -- Peer management
  mod config      -- Configuration
  mod home        -- Directory management
```

When adding features:

1. Keep code in the appropriate module
2. Add CLI commands in the main function
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
        let keypair = generate_keypair();
        assert_eq!(keypair.public.to_bytes().len(), 32);
    }
}
```

### Integration Tests

Test CLI commands manually:

```bash
# Test init flow
rad id
rad init --name "test" --description "Test project"
rad status

# Test patch flow
rad patch create --title "Test patch" --description "Testing"
rad patch list
```

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
- Follow Red Hat documentation style
- Use admonitions for important notes

---

## Pull Requests

### Process

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Run `cargo fmt` and `cargo clippy`
5. Update documentation if needed
6. Submit a pull request

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
- Rad version: [e.g., 0.1.0]
```

---

## License

By contributing, you agree that your contributions will be licensed under the same license as the project.

---

## Questions?

Open a GitHub Issue or Discussion for questions about contributing.
