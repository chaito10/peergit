# rad push

Push changes from the working copy to storage.

---

## Synopsis

```bash
rad push
```

## Description

`rad push` pushes commits from the current working copy to the bare repository in storage. This synchronizes the storage with your local changes.

---

## Examples

### Push Current Branch

```bash
# Make changes
echo "New feature" > feature.txt
git add feature.txt
git commit -m "Add new feature"

# Push to storage
rad push
```

### Push Output

```
Pushed to storage remote
  Branch: main
  HEAD:   52680c8de2dca294fa2482e8ef725a4cc1bb362c
```

---

## How It Works

1. Detect the current branch
2. Push commits to the `rad` remote (storage)
3. Update the branch ref in the database

---

## Branch Handling

| Scenario | Behavior |
|----------|----------|
| New commits | Pushes new commits to storage |
| Up to date | No changes pushed |
| Diverged | May fail if storage has diverged |

---

## Error Cases

### Repository Not Initialized

```
Error: Repository not initialized with Rad
```

Solution: Run `rad init` first.

### Push Rejected

```
Error: Push rejected - storage has diverged
```

Solution: Fetch and merge changes first:

```bash
git pull rad main
rad push
```

---

## See Also

- [rad fetch](fetch.md)
- [rad status](status.md)
- [rad init](init.md)
