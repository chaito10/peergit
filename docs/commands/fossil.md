# peergit fossil

Pass through to fossil CLI.

---

## Usage

```bash
peergit fossil <ARGS>...
```

---

## Description

Runs fossil commands directly, passing through all arguments. This is a convenience wrapper so you don't need to switch between `peergit` and `fossil` in your workflow.

### Example

```bash
# These are equivalent:
peergit fossil ui
fossil ui

peergit fossil status
fossil status

peergit fossil commit -m "message"
fossil commit -m "message"
```

---

## See Also

- [Fossil documentation](https://fossil-scm.org/)
