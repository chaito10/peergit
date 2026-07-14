# rad clone

Clone a repository from storage.

---

## Synopsis

```bash
rad clone <RID> [DIRECTORY]
```

## Description

`rad clone` creates a working copy of a repository from local storage. The repository must already be initialized with `rad init` or previously cloned.

---

## Arguments

| Argument | Description | Required |
|----------|-------------|----------|
| `RID` | Repository ID to clone | Yes |
| `DIRECTORY` | Target directory name | No (defaults to RID) |

---

## Examples

### Basic Clone

```bash
rad clone 1ca78dfe5991a24f5c18466be9a3fe8c998f4b141c07d3c4caa0b0ecacc7f319
```

### Clone to Specific Directory

```bash
rad clone 1ca78dfe5991a24f5c18466be9a3fe8c998f4b141c07d3c4caa0b0ecacc7f319 my-project
```

---

## Output

On success, the command outputs:

```
Repository cloned!
  RID:      1ca78dfe5991a24f5c18466be9a3fe8c998f4b141c07d3c4caa0b0ecacc7f319
  Directory: my-project
```

---

## How It Works

1. Look up the repository in local storage
2. Create the target directory
3. Initialize a git repository
4. Fetch all branches from the storage remote
5. Check out the default branch

---

## Remote Configuration

The cloned repository is configured with a `rad` remote pointing to storage:

```bash
git remote -v
# rad  /home/user/.local/share/radicle/storage/<rid> (fetch)
# rad  /home/user/.local/share/radicle/storage/<rid> (push)
```

---

## See Also

- [rad init](init.md)
- [rad push](push.md)
- [rad fetch](fetch.md)
