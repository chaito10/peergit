# rad init

Initialize a new Rad repository.

---

## Synopsis

```bash
rad init [OPTIONS]
```

## Description

`rad init` creates a new Rad repository in the current directory. It:

1. Generates an Ed25519 keypair (if one doesn't exist)
2. Creates an identity document with project metadata
3. Computes the Repository ID (RID)
4. Initializes a bare repository in storage
5. Pushes the working copy to storage
6. Creates namespace refs for patches
7. Stores metadata in the local database

---

## Options

| Option | Description | Default |
|--------|-------------|---------|
| `--name <NAME>` | Repository name | Directory name |
| `--description <DESC>` | Repository description | Empty |
| `--default-branch <BRANCH>` | Default branch name | `main` |
| `--visibility <VIS>` | Repository visibility (`public` or `private`) | `private` |

---

## Examples

### Basic Initialization

```bash
cd my-project
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

### With Custom Branch

```bash
rad init --name "my-project" --default-branch develop
```

### Public Repository

```bash
rad init --name "my-project" --visibility public
```

---

## Output

On success, the command outputs:

- **RID**: The unique repository identifier
- **Identity**: The public key in multibase format
- **DID**: The decentralized identifier
- **Branch**: The default branch name
- **Storage**: Path to the bare repository in storage

---

## Storage Structure

After initialization, the following structure is created:

```
$RAD_HOME/storage/<rid>/
  HEAD
  config
  objects/
  refs/
    heads/
      main
    patches/
  IDENTITY
```

---

## Environment Variables

| Variable | Description |
|----------|-------------|
| `RAD_HOME` | Override home directory |

---

## See Also

- [rad clone](clone.md)
- [rad status](status.md)
- [rad id](id.md)
