# Quick Start

This guide walks you through your first Rad session: generating an identity, initializing a repository, and creating a patch.

---

## Step 1: Generate an Identity

Every Rad user has an Ed25519 keypair that serves as their cryptographic identity.

```bash
rad id
```

Output:

```
Identity Information:
  Public Key: z6MkmSfm58EqKuNBqAFJcnVqETiCSW5F3t4A5HarBw6pF9km
  DID:        did:key:z6MkmSfm58EqKuNBqAFJcnVqETiCSW5F3t4A5HarBw6pF9km
  Key Path:   /home/user/.local/share/radicle/keys/radicle
```

!!! info "What happened?"
    Rad generated a new Ed25519 keypair and stored it in your home directory. The public key is displayed in multibase format (prefixed with `z`). The DID uses the `did:key` method for decentralized identification.

---

## Step 2: Initialize a Repository

Navigate to an existing Git repository and initialize it with Rad:

```bash
cd my-project

# If not already a git repo
git init
git add .
git commit -m "Initial commit"

# Initialize with Rad
rad init --name "my-project" --description "A collaborative project"
```

Output:

```
Repository initialized successfully!
  RID:      1ca78dfe5991a24f5c18466be9a3fe8c998f4b141c07d3c4caa0b0ecacc7f319
  Identity: z6MkmSfm58EqKuNBqAFJcnVqETiCSW5F3t4A5HarBw6pF9km
  DID:      did:key:z6MkmSfm58EqKuNBqAFJcnVqETiCSW5F3t4A5HarBw6pF9km
  Branch:   main
  Storage:  /home/user/.local/share/radicle/storage/1ca78...
```

!!! note "Repository ID (RID)"
    The RID is a SHA-256 hash of the identity document, encoded as a hex string. It uniquely identifies your repository across the network.

---

## Step 3: Check Status

```bash
rad status
```

Output:

```
Repository Status:
  Branch:  main
  HEAD:    52680c8de2dca294fa2482e8ef725a4cc1bb362c
  Remotes:
    - rad
  Rad URL: rad://1ca78dfe5991a24f5c18466be9a3fe8c998f4b141c07d3c4caa0b0ecacc7f319
```

---

## Step 4: Create a Patch

A patch is a proposed change to the repository, similar to a pull request.

```bash
rad patch create --title "Add README" --description "Adds project documentation"
```

Output:

```
Patch created!
  ID:    df5b5010-e2d9-4fe8-864d-735298aabc76
  Title: Add README
  Repo:  1ca78dfe5991a24f5c18466be9a3fe8c998f4b141c07d3c4caa0b0ecacc7f319
```

---

## Step 5: List Patches

```bash
rad patch list
```

Output:

```
ID                                     TITLE                AUTHOR               STATE
--------------------------------------------------------------------------------------------------------------
df5b5010                               Add README           z6MkmSfm...          open
```

---

## Step 6: Merge a Patch

After review, merge the patch:

```bash
rad patch merge df5b5010-e2d9-4fe8-864d-735298aabc76
```

Output:

```
Patch merged!
  ID:    df5b5010-e2d9-4fe8-864d-735298aabc76
  Title: Add README
```

---

## Step 7: Add a Peer

Add a collaborator's public key:

```bash
rad peer add z6MkrLMMsiPWUcNPHcRajuMi9mDfYckSoJyPwwnknocNYPm7 --alias alice
```

Output:

```
Peer added: z6MkrLMMsiPWUcNPHcRajuMi9mDfYckSoJyPwwnknocNYPm7
  Alias:    alice
```

List known peers:

```bash
rad peer list
```

---

## Step 8: View Configuration

```bash
rad config show
```

This displays the current configuration as JSON, including default seed nodes and network settings.

---

## Summary

You have learned how to:

1. **Generate** an Ed25519 identity with `rad id`
2. **Initialize** a repository with `rad init`
3. **Check** repository status with `rad status`
4. **Create** patches with `rad patch create`
5. **List** patches with `rad patch list`
6. **Merge** patches with `rad patch merge`
7. **Add** peers with `rad peer add`
8. **View** configuration with `rad config show`

---

## Next Steps

- [Commands Reference](commands/index.md) -- Complete command documentation
- [Concepts](concepts/identity.md) -- Understand Rad's core concepts
- [Configuration](configuration.md) -- Customize your installation
- [Architecture](architecture.md) -- Understand the system design
