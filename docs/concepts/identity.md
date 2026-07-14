# Identity

Rad uses Ed25519 public-key cryptography to establish decentralized identities.

---

## Overview

Every Rad user has a cryptographic identity consisting of:

- **Ed25519 Keypair**: Public and private keys for signing and verification
- **Public Key**: Encoded in multibase format (Radicle-compatible)
- **DID**: `did:key` identifier derived from the public key
- **Identity Document**: W3C-compliant document with keys and services

---

## Public Key Format

Rad public keys are 32-byte Ed25519 keys encoded in multibase Base32Z:

```
z6MkmSfm58EqKuNBqAFJcnVqETiCSW5F3t4A5HarBw6pF9km
```

!!! info "Multibase Encoding"
    Multibase encodes the version byte and key material together, making the format self-describing. The `z` prefix indicates Base32Z encoding.

### Generation

```bash
rad id
```

This generates a new keypair (if one doesn't exist) and displays the public key.

### Storage

Keys are stored in:

```
$RAD_HOME/keys/
  radicle          # Secret key (hex)
  radicle.pub      # Public key (hex)
```

!!! warning "Secret Key Security"
    The secret key is stored in plaintext for simplicity. In production, use a hardware security module (HSM) or encrypted storage.

---

## Decentralized Identifiers (DIDs)

DIDs provide a self-sovereign identity that doesn't depend on any central authority.

### DID Format

```
did:key:z6MkmSfm58EqKuNBqAFJcnVqETiCSW5F3t4A5HarBw6pF9km
```

| Component | Description |
|-----------|-------------|
| `did:` | DID method prefix |
| `key:` | DID method (key-based) |
| `z6Mk...` | Multibase-encoded public key |

### DID Document

The DID document is a W3C-compliant JSON-LD document:

```json
{
  "@context": ["https://www.w3.org/ns/did/v1", "https://w3id.org/security/suites/ed25519-2020/v1"],
  "id": "did:key:z6MkmSfm58EqKuNBqAFJcnVqETiCSW5F3t4A5HarBw6pF9km",
  "alsoKnownAs": ["did:key:z6MkmSfm58EqKuNBqAFJcnVqETiCSW5F3t4A5HarBw6pF9km"],
  "publicKey": [{
    "id": "did:key:z6Mk...#z6Mk...",
    "type": "Ed25519VerificationKey2018",
    "controller": "did:key:z6Mk...",
    "publicKeyMultibase": "z6Mk..."
  }],
  "service": [{
    "id": "did:key:z6Mk...#radicle-node",
    "type": "RadicleNode",
    "serviceEndpoint": "https://seed.radicle.xyz:8776"
  }]
}
```

### DID Resolution

To resolve a DID to a DID document:

1. Extract the multibase-encoded public key from the DID
2. Decode to raw bytes
3. Construct the DID document with keys and services

---

## Identity Documents

Identity documents extend the DID document with Radicle-specific metadata.

### Structure

```rust
pub struct IdentityDocument {
    pub id: Did,
    #[serde(rename = "alsoKnownAs")]
    pub also_known_as: Vec<Did>,
    pub keys: Vec<DocumentKey>,
    #[serde(rename = "service")]
    pub services: Vec<Service>,
}
```

### Storage

Identity documents are stored in:

```
$RAD_HOME/storage/<rid>/IDENTITY
```

The file contains the JSON-serialized identity document.

---

## Project Identity

When you initialize a repository with `rad init`, Rad creates a project identity:

### Project Metadata

```json
{
  "name": "my-project",
  "description": "A collaborative project",
  "defaultBranch": "main",
  "visibility": "private"
}
```

### Identity Document (Project)

The project identity document includes:

```json
{
  "@context": ["https://www.w3.org/ns/did/v1", "..."],
  "id": "did:key:z6Mk...",
  "alsoKnownAs": ["did:key:z6Mk..."],
  "keys": [{
    "key": "z6Mk...",
    "roles": ["keyMaintenance", "repoManagement"]
  }],
  "services": [{
    "id": "did:key:z6Mk...#radicle-node",
    "type": "RadicleNode",
    "serviceEndpoint": "https://seed.radicle.xyz:8776"
  }]
}
```

---

## Signing and Verification

Rad uses Ed25519 signatures for:

- **Announcements**: Prove repository ownership
- **Patches**: Prove patch authorship
- **Inventory**: Prove peer identity

### Signing

```rust
use ed25519_dalek::{Keypair, Signer};

let keypair = Keypair::from_bytes(&secret_key)?;
let message = b"repository announcement";
let signature = keypair.sign(message);
```

### Verification

```rust
use ed25519_dalek::{PublicKey, Verifier};

let public_key = PublicKey::from_bytes(&pub_bytes)?;
let message = b"repository announcement";
public_key.verify(message, &signature)?;
```

---

## Security Considerations

!!! warning "Production Use"
    Rad is a minimal reference implementation. For production use:

    - Store secret keys in encrypted storage or HSMs
    - Use transport encryption for network communication
    - Implement proper key rotation mechanisms
    - Verify signatures before trusting received data
