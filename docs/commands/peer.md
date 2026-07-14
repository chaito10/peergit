# rad peer

Manage known peers.

---

## Synopsis

```bash
rad peer <SUBCOMMAND>
```

## Subcommands

| Subcommand | Description |
|------------|-------------|
| `add` | Add a known peer |
| `list` | List known peers |
| `remove` | Remove a peer |

---

## rad peer add

Add a peer's public key to the local database.

### Synopsis

```bash
rad peer add <PUBLIC_KEY> [--alias <NAME>]
```

### Arguments

| Argument | Description | Required |
|----------|-------------|----------|
| `PUBLIC_KEY` | Multibase-encoded public key | Yes |
| `--alias <NAME>` | Human-readable alias | No |

### Examples

```bash
# Add peer with alias
rad peer add z6MkrLMMsiPWUcNPHcRajuMi9mDfYckSoJyPwwnknocNYPm7 --alias alice

# Add peer without alias
rad peer add z6Mkmqogy2qEM2ummccUthFEaaHvyYmYBYh3dbe9W4ebScxo
```

### Output

```
Peer added: z6MkrLMMsiPWUcNPHcRajuMi9mDfYckSoJyPwwnknocNYPm7
  Alias: alice
```

---

## rad peer list

List all known peers.

### Synopsis

```bash
rad peer list
```

### Examples

```bash
rad peer list
```

### Output

```
Public Key                              Alias      Added
----------------------------------------------------------------
z6MkrLMMsiPWUcNPHcRajuMi9mDfYckSo...  alice      2024-01-15
z6Mkmqogy2qEM2ummccUthFEaaHvyYmY...  bob        2024-01-16
```

---

## rad peer remove

Remove a peer from the local database.

### Synopsis

```bash
rad peer remove <PUBLIC_KEY>
```

### Arguments

| Argument | Description | Required |
|----------|-------------|----------|
| `PUBLIC_KEY` | Multibase-encoded public key | Yes |

### Examples

```bash
rad peer remove z6MkrLMMsiPWUcNPHcRajuMi9mDfYckSoJyPwwnknocNYPm7
```

### Output

```
Peer removed: z6MkrLMMsiPWUcNPHcRajuMi9mDfYckSoJyPwwnknocNYPm7
```

---

## Database Schema

Peers are stored in the `peers` table:

```sql
CREATE TABLE peers (
    public_key TEXT PRIMARY KEY,
    alias TEXT,
    node_id TEXT,
    added_at INTEGER NOT NULL
);
```

---

## See Also

- [rad id](id.md)
- [Configuration](../configuration.md)
