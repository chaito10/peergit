# rad patch

Manage patches (proposed changes).

---

## Synopsis

```bash
rad patch <SUBCOMMAND>
```

## Subcommands

| Subcommand | Description |
|------------|-------------|
| `create` | Create a new patch |
| `list` | List patches |
| `merge` | Merge a patch |

---

## rad patch create

Create a new patch from the current branch.

### Synopsis

```bash
rad patch create --title <TITLE> [--description <DESC>]
```

### Options

| Option | Description | Required |
|--------|-------------|----------|
| `--title <TITLE>` | Short description of changes | Yes |
| `--description <DESC>` | Detailed explanation | No |

### Examples

```bash
# Create patch from current branch
rad patch create --title "Add new feature" --description "Implements X"

# Create patch with description
rad patch create --title "Fix bug" --description "Resolves issue #42"
```

### Output

```
Patch created!
  ID:    df5b5010-e2d9-4fe8-864d-735298aabc76
  Title: Add new feature
  Repo:  1ca78dfe5991a24f5c18466be9a3fe8c998f4b141c07d3c4caa0b0ecacc7f319
```

### How It Works

1. Read the current branch and HEAD commit
2. Generate a unique patch ID (UUID)
3. Create a patch entry in the database
4. Create a ref at `refs/patches/<uuid>`
5. Return the patch ID

---

## rad patch list

List all patches for the current repository.

### Synopsis

```bash
rad patch list [--state <STATE>]
```

### Options

| Option | Description | Default |
|--------|-------------|---------|
| `--state <STATE>` | Filter by state (`open`, `merged`, `closed`) | All |

### Examples

```bash
# List all patches
rad patch list

# List open patches only
rad patch list --state open

# List merged patches
rad patch list --state merged
```

### Output

```
ID                                     TITLE                AUTHOR               STATE
--------------------------------------------------------------------------------------------------------------
df5b5010-e2d9-4fe8-864d-735298aabc76  Add new feature      z6MkmSfm...          open
a1b2c3d4-e5f6-7890-abcd-ef1234567890  Fix bug              z6MkrLMM...          merged
```

---

## rad patch merge

Merge a patch into the target branch.

### Synopsis

```bash
rad patch merge <PATCH_ID>
```

### Arguments

| Argument | Description | Required |
|----------|-------------|----------|
| `PATCH_ID` | UUID of the patch to merge | Yes |

### Examples

```bash
rad patch merge df5b5010-e2d9-4fe8-864d-735298aabc76
```

### Output

```
Patch merged!
  ID:    df5b5010-e2d9-4fe8-864d-735298aabc76
  Title: Add new feature
```

### How It Works

1. Look up the patch in the database
2. Verify the patch exists and is `open`
3. Merge the patch's HEAD commit into the target branch
4. Update the patch state to `merged`
5. Return success

### Error Cases

#### Patch Not Found

```
Error: Patch not found
```

#### Patch Already Merged

```
Error: Patch is not in 'open' state
```

#### Merge Conflict

```
Error: Merge failed - target branch has diverged
```

Solution: Manually rebase and try again.

---

## Patch Database Schema

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

---

## Patch Refs

Patches are stored as Git refs:

```
refs/patches/<uuid>
```

You can view patch refs with:

```bash
git show-ref | grep patches
```

---

## See Also

- [Patches Concepts](../concepts/patches.md)
- [rad status](status.md)
- [rad push](push.md)
