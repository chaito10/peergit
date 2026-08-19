# Repositories

PeerGit manages Fossil repositories with additional metadata for decentralized collaboration.

---

## Overview

A PeerGit repository consists of:

- **Fossil Repository**: Standard Fossil repository with all SCM state
- **RID**: SHA256-based Repository ID for P2P identification
- **Metadata**: Name, path, advertised status
- **Peer Information**: Who published this repository

---

## Repository ID (RID)

Each published repository has a unique identifier derived from the repository path and name:

```
a1b2c3d4e5f6...
```

### Generation

The RID is a SHA-256 hash of the repository path and name:

```rust
let rid = sha256(format!("{}:{}", path, name));
```

### Format

The RID is a 64-character hexadecimal string (32 bytes).

---

## Publishing a Repository

### Command

```bash
peergit repo publish --path ./my-project.fossil --name my-project
```

### Process

1. Verify the Fossil repository exists
2. Compute RID from path + name
3. Store in SQLite database
4. Mark as advertised for P2P discovery

### Output

```
Repository published:
  Name: my-project
  RID:  a1b2c3d4e5f6...
  Path: /path/to/my-project.fossil
```

---

## Listing Repositories

### Command

```bash
peergit repo list
```

### Output

```
Published repositories:
  my-project
    RID:    a1b2c3d4e5f6...
    Path:   /path/to/my-project.fossil
    Added:  2026-08-19
```

---

## Repository Database

### Schema

```sql
CREATE TABLE repositories (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    path TEXT NOT NULL
);

CREATE TABLE advertised_repos (
    rid TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    path TEXT NOT NULL,
    added_at INTEGER NOT NULL
);
```

---

## Fossil Integration

PeerGit does not manage Fossil repositories directly. Fossil handles:

- Repository initialization (`fossil init`)
- Commit history and branching
- Merge conflict resolution
- Wiki, tickets, and other SCM features

PeerGit adds:

- P2P transport via `--transport-command`
- Peer discovery and identity
- Encrypted transport via Noise
- Web dashboard for monitoring

---

## Unpublishing a Repository

To stop sharing a repository:

```bash
peergit repo unpublish --rid <RID>
```

---

## Visibility

| Visibility | Description |
|------------|-------------|
| `public` | Discoverable by all peers |
| `private` | Only shared with explicitly added peers |

!!! note "Current Implementation"
    Visibility is stored in metadata. Enforcement is planned for future versions.
