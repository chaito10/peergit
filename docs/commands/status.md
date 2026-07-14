# rad status

Show repository status.

---

## Synopsis

```bash
rad status
```

## Description

`rad status` displays the current state of a Rad repository, including the branch, HEAD commit, and configured remotes.

---

## Examples

### Show Status

```bash
rad status
```

### Output

```
Repository Status:
  Branch:  main
  HEAD:    52680c8de2dca294fa2482e8ef725a4cc1bb362c
  Remotes:
    - rad
  Rad URL: rad://1ca78dfe5991a24f5c18466be9a3fe8c998f4b141c07d3c4caa0b0ecacc7f319
```

---

## Output Fields

| Field | Description |
|-------|-------------|
| `Branch` | Current branch name |
| `HEAD` | Current HEAD commit hash |
| `Remotes` | Configured remotes |
| `Rad URL` | Radicle URL for the repository |

---

## Error Cases

### Not in a Git Repository

```
Error: Not in a git repository
```

### Repository Not Initialized

```
Error: Repository not initialized with Rad
```

Solution: Run `rad init` first.

---

## See Also

- [rad init](init.md)
- [rad push](push.md)
- [rad fetch](fetch.md)
