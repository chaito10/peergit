# peergit identity

Show node identity (DID, PeerId, public key).

---

## Usage

```bash
peergit identity
```

---

## Description

Displays the current node's cryptographic identity:

- **DID**: Decentralized Identifier (`did:key:z6Mk...`)
- **PeerId**: libp2p peer identifier (`12D3KooW...`)
- **Public Key**: Multibase-encoded Ed25519 public key
- **Key Path**: Path to the secret key file

---

## Output

```
Node Identity:
  DID:        did:key:z6Mk...
  PeerId:     12D3KooW...
  Public Key: 6C4X...
  Key Path:   C:\Users\you\AppData\Local\peergit\keys\node
```

---

## See Also

- [peergit init](init.md) -- Initialize a new identity
- [Identity Concept](../concepts/identity.md) -- Understanding identities
