# peergit repo

Repository management commands.

---

## peergit repo list

List published repositories.

### Usage

```bash
peergit repo list
```

### Output

```
Published repositories:
  my-project
    RID:    a1b2c3d4...
    Path:   /path/to/my-project.fossil
    Added:  2026-08-19
```

---

## peergit repo publish

Publish a local Fossil repository.

### Usage

```bash
peergit repo publish --path <PATH> --name <NAME>
```

### Options

| Option | Description |
|--------|-------------|
| `--path <PATH>` | Path to the Fossil repository file |
| `--name <NAME>` | Human-readable name for the repository |

### Example

```bash
peergit repo publish --path ./my-project.fossil --name my-project
```

### Description

1. Computes a SHA256-based Repository ID (RID)
2. Stores the mapping in the local SQLite database
3. Makes the repository discoverable by peers

---

## peergit repo discover

Discover a repository by RID or name.

### Usage

```bash
peergit repo discover --rid <RID>
peergit repo discover --name <NAME>
```

---

## peergit repo clone

Clone a published repository.

### Usage

```bash
peergit repo clone --rid <RID> --output <PATH>
```

---

## See Also

- [Repositories Concept](../concepts/repositories.md) -- Understanding repositories
- [Fossil Sync](../concepts/fossil-sync.md) -- Syncing over libp2p
