# Patches

!!! info "Planned Feature"
    Patch workflow is planned for a future version of PeerGit. Fossil handles patches via its built-in ticket and branch system.

---

## Overview

PeerGit currently delegates patch management to Fossil's native features:

- **Branches**: Create feature branches for patches
- **Tickets**: Use Fossil's ticket system for review workflows
- **Wiki**: Document changes in the wiki

---

## Planned Features

When patch support is added to PeerGit, it will include:

| Feature | Description |
|---------|-------------|
| **Patch Creation** | Create patches from branch commits |
| **Patch Listing** | List patches with status |
| **Patch Review** | Approve/reject patches |
| **Patch Merge** | Merge approved patches |
| **Patch Discussion** | Threaded comments on patches |

---

## Current Workflow

For now, use Fossil's built-in features:

```bash
# Create a feature branch
fossil branch new feature-xyz

# Make changes and commit
fossil commit -m "Add feature XYZ"

# Use Fossil's ticket system for review
fossil ticket new "Review: feature XYZ"

# Merge when approved
fossil merge feature-xyz
```

---

## See Also

- [Fossil documentation](https://fossil-scm.org/) -- Fossil's branch and ticket system
- [Fossil Sync](fossil-sync.md) -- Syncing over libp2p
