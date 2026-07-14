# rad sync

Synchronize repositories.

---

## Synopsis

```bash
rad sync
```

## Description

`rad sync` synchronizes local repositories with the storage backend. This command updates all repository metadata and ensures consistency.

!!! info "Planned Feature"
    Full synchronization with peers is planned but not yet implemented. Currently, this command provides a local sync operation.

---

## Examples

### Sync Repositories

```bash
rad sync
```

### Output

```
Sync complete!
  Repositories: 3
  Updated: 1
```

---

## How It Works

1. Scan all repositories in local storage
2. Update repository metadata in the database
3. Verify ref consistency
4. Report any changes

---

## Use Cases

### After Manual Storage Changes

If you modified the storage repository directly:

```bash
# Edit storage directly
git -C ~/.local/share/radicle/storage/<rid> commit --allow-empty -m "Update"

# Sync metadata
rad sync
```

### Before Backup

Ensure all metadata is consistent before backup:

```bash
rad sync
tar -czf radicle-backup.tar.gz ~/.local/share/radicle/
```

---

## Future Enhancements

- [ ] Peer synchronization
- [ ] Conflict detection and resolution
- [ ] Incremental sync
- [ ] Background sync process

---

## See Also

- [rad push](push.md)
- [rad fetch](fetch.md)
- [Architecture](../architecture.md)
