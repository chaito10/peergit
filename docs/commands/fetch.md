# rad fetch

Fetch changes from storage to the working copy.

---

## Synopsis

```bash
rad fetch
```

## Description

`rad fetch` fetches all refs from the bare repository in storage to your working copy. This updates your local branches with any changes that were pushed to storage.

---

## Examples

### Fetch Changes

```bash
rad fetch
```

### Fetch Output

```
Fetched from storage remote
  Branches updated: main
```

---

## How It Works

1. Connect to the `rad` remote (storage)
2. Fetch all refs
3. Update local branch references

---

## Use Cases

### Update After Storage Changes

If changes were made directly to the storage repository (e.g., via another clone):

```bash
# From another working copy
rad push

# In this working copy
rad fetch
git merge origin/main
```

### Before Creating a Patch

Ensure you have the latest changes before creating a patch:

```bash
rad fetch
rad patch create --title "New feature" --description "Adds X"
```

---

## Remote Configuration

The fetch uses the `rad` remote configured during `rad init`:

```bash
git remote -v
# rad  /home/user/.local/share/radicle/storage/<rid> (fetch)
```

---

## See Also

- [rad push](push.md)
- [rad status](status.md)
