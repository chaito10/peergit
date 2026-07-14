# rad id

Show identity information.

---

## Synopsis

```bash
rad id
```

## Description

`rad id` displays the current Rad identity, including the public key and DID. If no identity exists, it generates a new Ed25519 keypair.

---

## Examples

### Show Identity

```bash
rad id
```

### Output

```
Identity Information:
  Public Key: z6MkmSfm58EqKuNBqAFJcnVqETiCSW5F3t4A5HarBw6pF9km
  DID:        did:key:z6MkmSfm58EqKuNBqAFJcnVqETiCSW5F3t4A5HarBw6pF9km
  Key Path:   /home/user/.local/share/radicle/keys/radicle
```

### Generate New Identity

If no identity exists, `rad id` generates one:

```
Generated new identity!
  Public Key: z6MkmSfm58EqKuNBqAFJcnVqETiCSW5F3t4A5HarBw6pF9km
  DID:        did:key:z6MkmSfm58EqKuNBqAFJcnVqETiCSW5F3t4A5HarBw6pF9km
  Key Path:   /home/user/.local/share/radicle/keys/radicle
```

---

## Output Fields

| Field | Description |
|-------|-------------|
| `Public Key` | Multibase-encoded Ed25519 public key |
| `DID` | Decentralized Identifier (did:key) |
| `Key Path` | Path to the secret key file |

---

## Key Storage

Keys are stored in:

```
$RAD_HOME/keys/
  radicle          # Secret key (hex)
  radicle.pub      # Public key (hex)
```

!!! warning "Security"
    The secret key is stored in plaintext for simplicity. In production, use encrypted storage or a hardware security module.

---

## Identity Format

### Public Key

The public key is encoded in multibase Base32Z:

```
z6MkmSfm58EqKuNBqAFJcnVqETiCSW5F3t4A5HarBw6pF9km
```

### DID

The DID uses the `did:key` method:

```
did:key:z6MkmSfm58EqKuNBqAFJcnVqETiCSW5F3t4A5HarBw6pF9km
```

---

## See Also

- [Identity Concepts](../concepts/identity.md)
- [rad init](init.md)
