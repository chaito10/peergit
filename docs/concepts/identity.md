# Identity

PeerGit uses Ed25519 public-key cryptography to establish decentralized identities.

---

## Overview

Every PeerGit node has a cryptographic identity consisting of:

- **Ed25519 Keypair**: Public and private keys for signing and verification
- **Public Key**: Encoded in multibase format (Base32Z)
- **DID**: `did:key` identifier derived from the public key
- **PeerId**: libp2p peer identifier derived from the public key

---

## Public Key Format

PeerGit public keys are 32-byte Ed25519 keys encoded in multibase Base32Z:

```
6C4X...
```

!!! info "Multibase Encoding"
    Multibase encodes the version byte and key material together, making the format self-describing. The prefix indicates the encoding scheme.

### Generation

```bash
peergit init
```

This generates a new keypair (if one doesn't exist) and displays the public key.

### Storage

Keys are stored in:

```
$PEERGIT_HOME/keys/
  node          # Ed25519 secret key
```

!!! warning "Secret Key Security"
    The secret key is stored in plaintext for simplicity. In production, use encrypted storage.

---

## Decentralized Identifiers (DIDs)

DIDs provide a self-sovereign identity that doesn't depend on any central authority.

### DID Format

```
did:key:z6Mk...
```

| Component | Description |
|-----------|-------------|
| `did:` | DID method prefix |
| `key:` | DID method (key-based) |
| `z6Mk...` | Multibase-encoded public key |

### DID Resolution

To resolve a DID to a public key:

1. Extract the multibase-encoded public key from the DID
2. Decode to raw bytes
3. Verify the Ed25519 public key

---

## libp2p PeerId

PeerGit also generates a libp2p PeerId from the same Ed25519 key:

```
12D3KooW...
```

The PeerId is used by libp2p for:

- Peer identification in the DHT
- Connection authentication
- Kademlia key routing

### Converting Between Formats

```rust
// Public key -> PeerId
let peer_id = PeerId::from(keypair.public());

// PeerId -> Public key
let public_key = peer_id.extract_publickey()?;
```

---

## Signing and Verification

PeerGit uses Ed25519 signatures for:

- **Identity verification**: Prove ownership of a peer identity
- **Repository ownership**: Sign published repository metadata
- **Transport security**: Authenticate peers during Noise handshake

### Signing

```rust
use ed25519_dalek::{Keypair, Signer};

let keypair = Keypair::from_bytes(&secret_key)?;
let message = b"repository metadata";
let signature = keypair.sign(message);
```

### Verification

```rust
use ed25519_dalek::{PublicKey, Verifier};

let public_key = PublicKey::from_bytes(&pub_bytes)?;
let message = b"repository metadata";
public_key.verify(message, &signature)?;
```

---

## Security Considerations

!!! warning "Production Use"
    For production use:

    - Store secret keys in encrypted storage or HSMs
    - Transport encryption is handled by Noise (built into libp2p)
    - Verify public keys through a trusted channel out-of-band
    - Consider key rotation mechanisms
