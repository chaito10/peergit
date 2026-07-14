# Repositories

Rad manages Git repositories with additional metadata for decentralized collaboration.

---

## Overview

A Rad repository consists of:

- **Git Repository**: Standard Git repository with branches and commits
- **Identity**: Ed25519 keypair and DID document
- **Metadata**: Name, description, default branch, visibility
- **Namespace**: Refs for patches and other collaborative objects

---

## Repository ID (RID)

Each repository has a unique identifier derived from the identity document:

```
1ca78dfe5991a24f5c18466be9a3fe8c998f4b141c07d3c4caa0b0ecacc7f319
```

### Generation

The RID is a SHA-256 hash of the serialized identity document:

```rust
let identity_json = serde_json::to_string(&identity_document)?;
let rid = sha256(identity_json.as_bytes());
```

### Format

The RID is a 64-character hexadecimal string (32 bytes).

!!! info "Comparison with Heartwood"
    Heartwood uses SHA-1 OID (Git-compatible) for RIDs. Rad uses SHA-256 for simplicity, but this makes RIDs incompatible with the full Radicle network.

---

## Repository Storage

Repositories are stored in two locations:

### Bare Repository (Storage)

```
$RAD_HOME/storage/<rid>/
  HEAD
  config
  objects/
  refs/
    heads/
    patches/
  IDENTITY
```

### Working Copy (Pushed)

When you run `rad init` or `rad push`, Rad creates a bare repository in storage and pushes the working copy:

```bash
# Storage location
$RAD_HOME/storage/<rid>

# Working copy (standard git)
/path/to/your/project
```

---

## Repository Initialization

### Command

```bash
rad init --name "my-project" --description "A collaborative project"
```

### Process

1. Generate or load existing keypair
2. Create identity document with project metadata
3. Compute RID (SHA-256 of identity document)
4. Create bare repository in storage
5. Push working copy to storage remote
6. Create namespace refs (heads/*, patches/*)
7. Store metadata in SQLite

### Output

```
Repository initialized successfully!
  RID:      1ca78dfe5991a24f5c18466be9a3fe8c998f4b141c07d3c4caa0b0ecacc7f319
  Identity: z6MkmSfm58EqKuNBqAFJcnVqETiCSW5F3t4A5HarBw6pF9km
  DID:      did:key:z6MkmSfm58EqKuNBqAFJcnVqETiCSW5F3t4A5HarBw6pF9km
  Branch:   main
  Storage:  /home/user/.local/share/radicle/storage/1ca78...
```

---

## Repository Cloning

### Command

```bash
rad clone <rid>
```

### Process

1. Look up repository in local storage
2. Create working directory
3. Initialize git repository
4. Fetch all branches from storage

---

## Push and Fetch

### Push

```bash
rad push
```

Pushes changes from the working copy to storage:

1. Detect current branch
2. Push commits to storage remote

### Fetch

```bash
rad fetch
```

Fetches changes from storage to working copy:

1. Fetch all refs from storage remote
2. Update local branches

---

## Repository Metadata

### Database Schema

```sql
CREATE TABLE repositories (
    id TEXT PRIMARY KEY,        -- RID
    name TEXT NOT NULL,
    description TEXT,
    default_branch TEXT,
    identity TEXT NOT NULL,     -- JSON identity document
    created_at INTEGER NOT NULL
);
```

### Access

```bash
# View repository info
rad status

# List all repositories
rad status --all
```

---

## Namespace Refs

Repositories use namespace refs for collaborative objects:

### Structure

```
refs/
  heads/
    main
    feature/*
  patches/
    <uuid>/
      meta        -- Patch metadata
      <version>   -- Patch commits
```

### Patch Refs

Patches are stored as refs:

```
refs/patches/<uuid>/meta
refs/patches/<uuid>/v0
refs/patches/<uuid>/v1
```

See [Patches](patches.md) for details.

---

## Remote Configuration

### Storage Remote

When you initialize a repository, Rad adds a `rad` remote pointing to storage:

```bash
git remote -v
# rad  /home/user/.local/share/radicle/storage/<rid> (fetch)
# rad  /home/user/.local/share/radicle/storage/<rid> (push)
```

### Adding Remotes

```bash
git remote add peer1 /path/to/peer/storage/<rid>
```

---

## Visibility

Repositories can have different visibility levels:

| Visibility | Description |
|------------|-------------|
| `public` | Visible to all peers |
| `private` | Only visible to owner and collaborators |

!!! note "Current Implementation"
    Visibility is stored in metadata but not enforced in the current implementation.

---

## Future Enhancements

- [ ] Remote storage synchronization
- [ ] Repository cloning from peers
- [ ] Access control lists
- [ ] Repository archiving
