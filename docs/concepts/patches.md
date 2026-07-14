# Patches

Patches are proposed changes to a repository, similar to pull requests in centralized systems.

---

## Overview

A patch in Rad consists of:

- **Metadata**: Title, description, author, timestamps
- **Commits**: One or more commits representing the proposed changes
- **Reviews**: Approval or rejection from reviewers (planned)
- **Discussion**: Threaded comments (planned)

---

## Patch Lifecycle

### States

| State | Description |
|-------|-------------|
| `open` | Patch is under review |
| `merged` | Patch has been merged into the target branch |
| `closed` | Patch was rejected or withdrawn |

### Workflow

```
Create patch (open)
    ↓
Review and discuss
    ↓
Approve patch
    ↓
Merge into target branch
    ↓
Patch merged
```

---

## Creating a Patch

### Command

```bash
rad patch create --title "Add new feature" --description "Implements X"
```

### Process

1. Read current branch and HEAD commit
2. Generate unique patch ID (UUID)
3. Create patch entry in database
4. Create refs/patches/<uuid> ref
5. Return patch ID

### Output

```
Patch created!
  ID:    df5b5010-e2d9-4fe8-864d-735298aabc76
  Title: Add new feature
  Repo:  1ca78dfe5991a24f5c18466be9a3fe8c998f4b141c07d3c4caa0b0ecacc7f319
```

---

## Listing Patches

### Command

```bash
rad patch list
```

### Output

```
ID                                     TITLE                AUTHOR               STATE
--------------------------------------------------------------------------------------------------------------
df5b5010-e2d9-4fe8-864d-735298aabc76  Add new feature      z6MkmSfm...          open
a1b2c3d4-e5f6-7890-abcd-ef1234567890  Fix bug              z6MkrLMM...          merged
```

---

## Patch Database

### Schema

```sql
CREATE TABLE patches (
    id TEXT PRIMARY KEY,
    rid TEXT NOT NULL,
    title TEXT NOT NULL,
    description TEXT NOT NULL,
    author TEXT NOT NULL,
    state TEXT NOT NULL,
    target TEXT NOT NULL,
    head TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    FOREIGN KEY (rid) REFERENCES repositories(id)
);
```

### Fields

| Field | Type | Description |
|-------|------|-------------|
| `id` | TEXT | UUID identifying the patch |
| `rid` | TEXT | Repository ID |
| `title` | TEXT | Short description of changes |
| `description` | TEXT | Detailed explanation |
| `author` | TEXT | Public key of author |
| `state` | TEXT | `open`, `merged`, or `closed` |
| `target` | TEXT | Target branch (default: main) |
| `head` | TEXT | HEAD commit hash |
| `created_at` | INTEGER | Creation timestamp |
| `updated_at` | INTEGER | Last update timestamp |

---

## Patch Refs

Patches are stored as Git refs:

```
refs/
  patches/
    <uuid>/
      meta        -- Patch metadata (not used yet)
      v0          -- Original version
      v1          -- Updated version (if amended)
```

### Ref Format

```bash
git show-ref
# df5b5010-e2d9-4fe8-864d-735298aabc76 refs/patches/df5b5010-e2d9-4fe8-864d-735298aabc76
```

---

## Merging a Patch

### Command

```bash
rad patch merge <patch_id>
```

### Process

1. Look up patch in database
2. Verify patch exists and is open
3. Merge HEAD commit into target branch
4. Update patch state to `merged`
5. Return success

### Output

```
Patch merged!
  ID:    df5b5010-e2d9-4fe8-864d-735298aabc76
  Title: Add new feature
```

### Merge Strategy

Currently, Rad uses a simple fast-forward merge. If the target branch has diverged, the merge will fail.

!!! warning "Merge Conflicts"
    Rad does not currently handle merge conflicts. If the target branch has new commits since the patch was created, you may need to rebase manually.

---

## Patch Metadata

### Identity

Each patch is identified by:

- **UUID**: Unique identifier (e.g., `df5b5010-e2d9-4fe8-864d-735298aabc76`)
- **RID**: Repository the patch belongs to
- **Author**: Public key of the creator

### Versioning

Patches can be amended by creating new versions:

```bash
# Amend a patch
git commit --amend
rad patch create --title "Updated feature" --description "Improved implementation"
```

!!! note "Version Tracking"
    The current implementation tracks only the latest version. Full version history is planned.

---

## Reviews and Discussion

!!! info "Planned Features"
    Reviews and discussion threads are planned but not yet implemented.

### Planned Review Types

| Type | Description |
|------|-------------|
| `approve` | Patch looks good to merge |
| `reject` | Patch needs changes |
| `comment` | General feedback |

### Planned Discussion

- Threaded comments on patches
- Inline code comments
- Review requests to specific peers

---

## Example Workflow

### 1. Create a Patch

```bash
# Make changes
echo "New feature" > feature.txt
git add feature.txt
git commit -m "Add new feature"

# Create patch
rad patch create --title "Add feature" --description "Adds a new feature file"
```

### 2. Review the Patch

```bash
# List patches
rad patch list

# View patch details
rad patch show df5b5010-e2d9-4fe8-864d-735298aabc76
```

### 3. Merge the Patch

```bash
# After approval
rad patch merge df5b5010-e2d9-4fe8-864d-735298aabc76
```

---

## Future Enhancements

- [ ] Patch versioning with full history
- [ ] Review workflow (approve/reject)
- [ ] Threaded discussion
- [ ] Inline code comments
- [ ] Review requests
- [ ] CI integration
- [ ] Merge conflict resolution
