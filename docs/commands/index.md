# Commands Reference

This section documents all available Rad commands.

---

## Overview

Rad provides a git-like CLI interface for distributed code collaboration.

### Usage Pattern

```bash
rad <command> [subcommand] [options]
```

### Global Options

| Option | Description |
|--------|-------------|
| `-h`, `--help` | Print help information |
| `-V`, `--version` | Print version information |

---

## Command Categories

### Identity

| Command | Description |
|---------|-------------|
| `rad id` | Show identity information |

### Repository Management

| Command | Description |
|---------|-------------|
| `rad init` | Initialize a Rad repository |
| `rad clone` | Clone a repository |
| `rad push` | Push changes to storage |
| `rad fetch` | Fetch changes from storage |
| `rad status` | Show repository status |

### Peer Management

| Command | Description |
|---------|-------------|
| `rad peer add` | Add a known peer |
| `rad peer list` | List known peers |

### Patches

| Command | Description |
|---------|-------------|
| `rad patch create` | Create a new patch |
| `rad patch list` | List patches |
| `rad patch merge` | Merge a patch |

### Configuration

| Command | Description |
|---------|-------------|
| `rad config show` | Show configuration |
| `rad config init` | Initialize configuration |

### Synchronization

| Command | Description |
|---------|-------------|
| `rad sync` | Sync repositories |

---

## Examples

```bash
# Generate identity
rad id

# Initialize repository
rad init --name "my-project" --description "A cool project"

# Check status
rad status

# Create patch
rad patch create --title "Add feature" --description "Implements X"

# Add peer
rad peer add z6MkrLMMsiPWUcNPHcRajuMi9mDfYckSoJyPwwnknocNYPm7 --alias alice

# Show config
rad config show
```

---

## Exit Codes

| Code | Description |
|------|-------------|
| 0 | Success |
| 1 | General error |
| 2 | Invalid arguments |
| 3 | Git operation failed |
| 4 | Database error |
| 5 | Identity not found |
| 6 | Repository not initialized |

---

## Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `RAD_HOME` | Override home directory | Platform-specific |
| `RUST_LOG` | Log level filter | `warn` |
